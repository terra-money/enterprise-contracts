use crate::distributing::query_enterprise_components;
use crate::repository::weights_repository::weights_repository_mut;
use common::cw::{Context, QueryContext};
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Deps, Response, StdError, StdResult, Uint128};
use cw_storage_plus::{Item, Map};
use enterprise_governance_api::msg::QueryMsg::TotalVotes;
use funds_distributor_api::api::DistributionType::Participation;
use funds_distributor_api::api::{
    NewProposalCreatedMsg, NumberProposalsTrackedResponse, ProposalIdsTrackedResponse,
};
use funds_distributor_api::error::DistributorResult;
use funds_distributor_api::response::execute_new_proposal_created_response;
use poll_engine_api::api::{PollId, TotalVotesParams, TotalVotesResponse};

// TODO: hide those storages behind an interface

pub const PROPOSALS_TRACKED: Item<u8> = Item::new("proposals_tracked");

// TODO: fill up in migration... or do we even offer it in migration? maybe just let them set N through a proposal
// TODO: use ProposalId instead of PollId
pub const PARTICIPATION_PROPOSAL_IDS: Map<PollId, ()> = Map::new("participation_proposal_ids");

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

    // TODO: is it unsafe to cast this? shouldn't be
    // TODO: should we fail if it is greater than?
    if (proposal_ids_tracked.len() as u8) == proposals_to_track {
        let (first_tracked, _) = PARTICIPATION_PROPOSAL_IDS.first(ctx.deps.storage)?.ok_or_else(|| StdError::generic_err("Invalid state - couldn't find first tracked proposal ID for participation rewards"))?;

        PARTICIPATION_PROPOSAL_IDS.remove(ctx.deps.storage, first_tracked);
    }
    PARTICIPATION_PROPOSAL_IDS.save(ctx.deps.storage, msg.proposal_id, &())?;

    // TODO: trigger a new era

    // TODO: optimize, we don't have to read it again
    let proposal_ids_tracked = get_proposal_ids_tracked(ctx.deps.as_ref())?;

    let components = query_enterprise_components(ctx.deps.as_ref())?;
    let total_votes_response: TotalVotesResponse = ctx.deps.querier.query_wasm_smart(
        components.enterprise_governance_contract.to_string(),
        &TotalVotes(TotalVotesParams {
            poll_ids: proposal_ids_tracked,
        }),
    )?;

    weights_repository_mut(ctx.deps.branch(), Participation)
        .set_total_weight(total_votes_response.total_votes)?;

    Ok(execute_new_proposal_created_response(msg.proposal_id))
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
        .collect::<StdResult<Vec<PollId>>>()?;

    Ok(proposal_ids)
}
