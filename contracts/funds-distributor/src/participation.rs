use crate::distributing::query_enterprise_components;
use crate::repository::era_repository::{
    get_current_era, get_user_first_era_with_weight, get_user_last_resolved_era, set_current_era,
    set_user_first_era_with_weight_if_empty,
};
use crate::repository::weights_repository::{weights_repository, weights_repository_mut};
use crate::state::{EraId, ADMIN};
use crate::user_weights::{initialize_user_indices, update_user_indices};
use common::cw::Order::Descending;
use common::cw::{Context, QueryContext};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Deps, DepsMut, Response, StdResult, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
use enterprise_governance_api::msg::QueryMsg::TotalVotes;
use enterprise_governance_controller_api::api::ProposalType::General;
use enterprise_governance_controller_api::api::{ProposalId, ProposalsParams, ProposalsResponse};
use funds_distributor_api::api::DistributionType::Participation;
use funds_distributor_api::api::{
    NewProposalCreatedMsg, NumberProposalsTrackedResponse, PreUserVotesChangeMsg,
    ProposalIdsTrackedResponse, UpdateNumberProposalsTrackedMsg,
};
use funds_distributor_api::error::DistributorError::Unauthorized;
use funds_distributor_api::error::DistributorResult;
use funds_distributor_api::response::{
    execute_new_proposal_created_response, execute_pre_user_votes_change_response,
    execute_update_number_proposals_tracked_response,
};
use poll_engine_api::api::{TotalVotesParams, TotalVotesResponse};

// TODO: hide those storages behind an interface

pub const NUMBER_PROPOSALS_TRACKED: Item<u8> = Item::new("proposals_tracked");

#[cw_serde]
/// A single proposal ID tracked within a distribution era.
struct TrackedParticipationProposal {
    pub era_id: EraId,
    pub proposal_id: ProposalId,
}

struct TrackedParticipationProposalIndexes<'a> {
    pub proposal: MultiIndex<'a, ProposalId, TrackedParticipationProposal, (EraId, ProposalId)>,
}

impl IndexList<TrackedParticipationProposal> for TrackedParticipationProposalIndexes<'_> {
    fn get_indexes(
        &'_ self,
    ) -> Box<dyn Iterator<Item = &'_ dyn Index<TrackedParticipationProposal>> + '_> {
        let v: Vec<&dyn Index<TrackedParticipationProposal>> = vec![&self.proposal];
        Box::new(v.into_iter())
    }
}

// TODO: fill up in migration... or do we even offer it in migration? maybe just let them set N through a proposal
#[allow(non_snake_case)]
fn TRACKED_PARTICIPATION_PROPOSALS<'a>() -> IndexedMap<
    'a,
    (EraId, ProposalId),
    TrackedParticipationProposal,
    TrackedParticipationProposalIndexes<'a>,
> {
    let indexes = TrackedParticipationProposalIndexes {
        proposal: MultiIndex::new(
            |_, tracked_proposal| tracked_proposal.proposal_id,
            "tracked_participation_proposals",
            "tracked_participation_proposals__proposal",
        ),
    };
    IndexedMap::new("tracked_participation_proposals", indexes)
}

fn save_tracked_proposal(deps: DepsMut, era_id: EraId, proposal_id: ProposalId) -> StdResult<()> {
    TRACKED_PARTICIPATION_PROPOSALS().save(
        deps.storage,
        (era_id, proposal_id),
        &TrackedParticipationProposal {
            era_id,
            proposal_id,
        },
    )
}

pub fn new_proposal_created(
    ctx: &mut Context,
    msg: NewProposalCreatedMsg,
) -> DistributorResult<Response> {
    // TODO: optimize this, we don't have to read through all of them
    let current_era = get_current_era(ctx.deps.as_ref(), Participation)?;
    // TODO: do we use current era here for real?
    let proposal_ids_tracked = get_proposal_ids_tracked(ctx.deps.as_ref(), current_era)?;

    let proposals_to_track = NUMBER_PROPOSALS_TRACKED.load(ctx.deps.storage)?;

    if proposals_to_track == 0 {
        return Ok(Response::new());
    }

    let next_era = current_era + 1;
    set_current_era(ctx.deps.branch(), next_era, Participation)?;

    // TODO: should we fail if it is greater than?
    if (proposal_ids_tracked.len() as u8) == proposals_to_track {
        // we are tracking maximum proposals, so we copy over all from the previous era excluding
        // the oldest proposal ID (lowest number)

        let mut first_tracked_proposal = None;

        // TODO: do we check here if tracked proposals are empty?
        for tracked_proposal in proposal_ids_tracked {
            if let Some(id) = first_tracked_proposal {
                if id > tracked_proposal {
                    save_tracked_proposal(ctx.deps.branch(), next_era, id)?;
                    first_tracked_proposal = Some(tracked_proposal)
                } else {
                    save_tracked_proposal(ctx.deps.branch(), next_era, tracked_proposal)?;
                }
            } else {
                first_tracked_proposal = Some(tracked_proposal)
            }
        }
    } else {
        // we aren't tracking enough proposals yet, just copy over all the ones from the previous era
        for id in proposal_ids_tracked {
            save_tracked_proposal(ctx.deps.branch(), next_era, id)?;
        }
    }

    save_tracked_proposal(ctx.deps.branch(), next_era, msg.proposal_id)?;

    // TODO: we know that the new proposal has 0 votes, just check if we removed one and remove its votes from previous era's total
    let total_votes = query_total_participation_weight(ctx.deps.as_ref(), next_era)?;

    weights_repository_mut(ctx.deps.branch(), Participation)
        .set_total_weight(total_votes, next_era)?;

    Ok(execute_new_proposal_created_response(msg.proposal_id))
}

pub fn query_total_participation_weight(deps: Deps, era_id: EraId) -> DistributorResult<Uint128> {
    // TODO: optimize, we don't have to read it again (this is called from contexts where we already have proposal IDs loaded)
    let proposal_ids_tracked = get_proposal_ids_tracked(deps, era_id)?;

    let components = query_enterprise_components(deps)?;
    let total_votes_response: TotalVotesResponse = deps.querier.query_wasm_smart(
        components.enterprise_governance_contract.to_string(),
        &TotalVotes(TotalVotesParams {
            poll_ids: proposal_ids_tracked,
        }),
    )?;

    Ok(total_votes_response.total_votes)
}

pub fn execute_update_number_proposals_tracked(
    ctx: &mut Context,
    msg: UpdateNumberProposalsTrackedMsg,
) -> DistributorResult<Response> {
    let admin = ADMIN.load(ctx.deps.storage)?;

    if ctx.info.sender != admin {
        return Err(Unauthorized);
    }

    let current_era = get_current_era(ctx.deps.as_ref(), Participation)?;
    let next_era = current_era + 1;

    set_current_era(ctx.deps.branch(), next_era, Participation)?;

    // TODO: we know part of them if we had N > 0 before, we can reuse them
    let new_tracked_proposal_ids: Vec<ProposalId> =
        get_last_n_general_proposal_ids(ctx.deps.as_ref(), msg.number_proposals_tracked)?;

    for proposal_id in new_tracked_proposal_ids {
        TRACKED_PARTICIPATION_PROPOSALS().save(
            ctx.deps.storage,
            (next_era, proposal_id),
            &TrackedParticipationProposal {
                era_id: next_era,
                proposal_id,
            },
        )?;
    }

    NUMBER_PROPOSALS_TRACKED.save(ctx.deps.storage, &msg.number_proposals_tracked)?;

    // TODO: store the new weights properly. this can also be improved later, if we just figure out the difference between old and new proposal weights
    // let new_total_weight = query_total_participation_weight(ctx.deps.as_ref())?;
    //
    // weights_repository_mut(ctx.deps.branch(), Participation).set_total_weight(new_total_weight)?;

    Ok(execute_update_number_proposals_tracked_response(
        msg.number_proposals_tracked,
    ))
}

pub fn pre_user_votes_change(
    ctx: &mut Context,
    msg: PreUserVotesChangeMsg,
) -> DistributorResult<Response> {
    let current_era = get_current_era(ctx.deps.as_ref(), Participation)?;

    // TODO: when there's multiple users, this will perform a query to gov contract for each user. we can optimize by introducing bulk query
    for user in msg.users {
        let user = ctx.deps.api.addr_validate(&user)?;

        let last_resolved_era =
            get_user_last_resolved_era(ctx.deps.as_ref(), user.clone(), Participation)?;

        let first_relevant_era = match last_resolved_era {
            Some(last_resolved_era) => Some(last_resolved_era + 1), // TODO: what if last resolved era is current era?
            None => get_user_first_era_with_weight(ctx.deps.as_ref(), user.clone(), Participation)?,
        };

        // TODO: resume here

        // TODO: we probably have to split handling of current era and past eras
        match first_relevant_era {
            Some(first_relevant_era) => {
                for era in first_relevant_era..=current_era {
                    let user_total_votes = weights_repository(ctx.deps.as_ref(), Participation)
                        .get_user_weight(user.clone(), era)?;
                    // TODO: we really shouldn't query it all for each era here
                    match user_total_votes {
                        // TODO: not sure if this initialize_user_indices works
                        // the reasoning is that we may have had N=0, user voted, N gets incremented to >0, we will then not get None here but
                        // we'll assume their indices have been initialized
                        None => {
                            if era == current_era {
                                initialize_user_indices(
                                    ctx.deps.branch(),
                                    user.clone(),
                                    Participation,
                                )?
                            }
                        }
                        Some(total) => update_user_indices(
                            ctx.deps.branch(),
                            user.clone(),
                            total,
                            Participation,
                        )?,
                    }
                }
            }
            None => {
                set_user_first_era_with_weight_if_empty(
                    ctx.deps.branch(),
                    user.clone(),
                    current_era,
                    Participation,
                )?;
                initialize_user_indices(ctx.deps.branch(), user.clone(), Participation)?;
            }
        }

        // TODO: we can optimize this for simple vote casts by just storing their last known participation weight,
        // TODO: querying their current vote amount on this proposal
        // TODO: and then just using their new weight to deduce the new weight
    }

    Ok(execute_pre_user_votes_change_response())
}

pub fn query_number_proposals_tracked(
    qctx: QueryContext,
) -> DistributorResult<NumberProposalsTrackedResponse> {
    let number_proposals_tracked = NUMBER_PROPOSALS_TRACKED.load(qctx.deps.storage)?;

    Ok(NumberProposalsTrackedResponse {
        number_proposals_tracked,
    })
}

pub fn query_proposal_ids_tracked(
    qctx: QueryContext,
) -> DistributorResult<ProposalIdsTrackedResponse> {
    let current_era = get_current_era(qctx.deps, Participation)?;
    let proposal_ids = get_proposal_ids_tracked(qctx.deps, current_era)?;

    Ok(ProposalIdsTrackedResponse { proposal_ids })
}

pub fn get_proposal_ids_tracked(deps: Deps, era_id: EraId) -> DistributorResult<Vec<u64>> {
    let proposal_ids = TRACKED_PARTICIPATION_PROPOSALS()
        .prefix(era_id)
        .range(deps.storage, None, None, Ascending)
        .map(|res| res.map(|(_, it)| it.proposal_id))
        .collect::<StdResult<Vec<ProposalId>>>()?;

    Ok(proposal_ids)
}

fn get_last_n_general_proposal_ids(deps: Deps, n: u8) -> DistributorResult<Vec<ProposalId>> {
    let components = query_enterprise_components(deps)?;

    let mut proposal_ids_found: Vec<ProposalId> = vec![];

    let mut start_after: Option<ProposalId> = None;

    loop {
        let remaining_ids_to_find = n - proposal_ids_found.len() as u8;

        if remaining_ids_to_find == 0u8 {
            break;
        }

        let proposals: ProposalsResponse = deps.querier.query_wasm_smart(
            components
                .enterprise_governance_controller_contract
                .to_string(),
            &enterprise_governance_controller_api::msg::QueryMsg::Proposals(ProposalsParams {
                filter: None,
                start_after,
                limit: Some((remaining_ids_to_find * 2) as u32),
                order: Some(Descending),
            }),
        )?;

        if proposals.proposals.is_empty() {
            break;
        }

        proposals
            .proposals
            .iter()
            .filter(|it| it.proposal.proposal_type == General)
            .take(remaining_ids_to_find as usize)
            .for_each(|it| proposal_ids_found.push(it.proposal.id));

        start_after = proposals.proposals.last().map(|it| it.proposal.id);
    }

    proposal_ids_found.reverse();

    Ok(proposal_ids_found)
}
