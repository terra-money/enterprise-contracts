use crate::distributing::query_enterprise_components;
use crate::repository::weights_repository::weights_repository_mut;
use crate::state::ADMIN;
use common::cw::Order::Descending;
use common::cw::{Context, QueryContext};
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Deps, Response, StdError, StdResult, Uint128};
use cw_storage_plus::{Item, Map};
use enterprise_governance_api::msg::QueryMsg::TotalVotes;
use enterprise_governance_controller_api::api::ProposalType::General;
use enterprise_governance_controller_api::api::{ProposalId, ProposalsParams, ProposalsResponse};
use funds_distributor_api::api::DistributionType::Participation;
use funds_distributor_api::api::{
    NewProposalCreatedMsg, NumberProposalsTrackedResponse, ProposalIdsTrackedResponse,
    UpdateNumberProposalsTrackedMsg,
};
use funds_distributor_api::error::DistributorError::Unauthorized;
use funds_distributor_api::error::DistributorResult;
use funds_distributor_api::response::{
    execute_new_proposal_created_response, execute_update_number_proposals_tracked_response,
};
use poll_engine_api::api::{TotalVotesParams, TotalVotesResponse};

// TODO: hide those storages behind an interface

pub const PROPOSALS_TRACKED: Item<u8> = Item::new("proposals_tracked");

// TODO: fill up in migration... or do we even offer it in migration? maybe just let them set N through a proposal
pub const PARTICIPATION_PROPOSAL_IDS: Map<ProposalId, ()> = Map::new("participation_proposal_ids");

pub const PARTICIPATION_TOTAL_WEIGHT: Item<Uint128> = Item::new("participation_total_weight");

pub fn new_proposal_created(
    ctx: &mut Context,
    msg: NewProposalCreatedMsg,
) -> DistributorResult<Response> {
    // TODO: optimize this, we don't have to read through all of them
    let proposal_ids_tracked = get_proposal_ids_tracked(ctx.deps.as_ref())?;

    let proposals_to_track = PROPOSALS_TRACKED.load(ctx.deps.storage)?;

    if proposals_to_track == 0 {
        return Ok(Response::new());
    }

    // TODO: should we fail if it is greater than?
    if (proposal_ids_tracked.len() as u8) == proposals_to_track {
        let (first_tracked, _) = PARTICIPATION_PROPOSAL_IDS.first(ctx.deps.storage)?.ok_or_else(|| StdError::generic_err("Invalid state - couldn't find first tracked proposal ID for participation rewards"))?;

        PARTICIPATION_PROPOSAL_IDS.remove(ctx.deps.storage, first_tracked);
    }
    PARTICIPATION_PROPOSAL_IDS.save(ctx.deps.storage, msg.proposal_id, &())?;

    // TODO: trigger a new era

    let total_votes = query_total_participation_weight(ctx.deps.as_ref())?;

    weights_repository_mut(ctx.deps.branch(), Participation).set_total_weight(total_votes)?;

    Ok(execute_new_proposal_created_response(msg.proposal_id))
}

fn query_total_participation_weight(deps: Deps) -> DistributorResult<Uint128> {
    // TODO: optimize, we don't have to read it again
    let proposal_ids_tracked = get_proposal_ids_tracked(deps)?;

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

    let old_number_tracked = PROPOSALS_TRACKED.load(ctx.deps.storage)?;

    // TODO: we know part of them if we had N > 0 before, we can reuse them
    let mut new_tracked_proposal_ids: Vec<ProposalId> =
        get_last_n_general_proposal_ids(ctx.deps.as_ref(), msg.number_proposals_tracked)?;

    PARTICIPATION_PROPOSAL_IDS.clear(ctx.deps.storage);
    for proposal_id in new_tracked_proposal_ids {
        PARTICIPATION_PROPOSAL_IDS.save(ctx.deps.storage, proposal_id, &())?;
    }

    // TODO: trigger a new era

    let new_total_weight = query_total_participation_weight(ctx.deps.as_ref())?;

    weights_repository_mut(ctx.deps.branch(), Participation).set_total_weight(new_total_weight)?;

    Ok(execute_update_number_proposals_tracked_response(
        old_number_tracked,
        msg.number_proposals_tracked,
    ))
}

pub fn query_number_proposals_tracked(
    qctx: QueryContext,
) -> DistributorResult<NumberProposalsTrackedResponse> {
    let number_proposals_tracked = PROPOSALS_TRACKED.load(qctx.deps.storage)?;

    Ok(NumberProposalsTrackedResponse {
        number_proposals_tracked,
    })
}

pub fn query_proposal_ids_tracked(
    qctx: QueryContext,
) -> DistributorResult<ProposalIdsTrackedResponse> {
    let proposal_ids = get_proposal_ids_tracked(qctx.deps)?;

    Ok(ProposalIdsTrackedResponse { proposal_ids })
}

pub fn get_proposal_ids_tracked(deps: Deps) -> DistributorResult<Vec<u64>> {
    let proposal_ids = PARTICIPATION_PROPOSAL_IDS
        .range(deps.storage, None, None, Ascending)
        .map(|res| res.map(|(id, _)| id))
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
