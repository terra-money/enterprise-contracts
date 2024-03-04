use common::cw::QueryContext;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Deps, StdResult, Uint128};
use cw_storage_plus::{Item, Map};
use funds_distributor_api::api::{NumberProposalsTrackedResponse, ProposalIdsTrackedResponse};
use funds_distributor_api::error::DistributorResult;
use poll_engine_api::api::PollId;

// TODO: hide those storages behind an interface

pub const PROPOSALS_TRACKED: Item<u8> = Item::new("proposals_tracked");

// TODO: fill up in migration... or do we even offer it in migration? maybe just let them set N through a proposal
// TODO: use ProposalId instead of PollId
pub const PARTICIPATION_PROPOSAL_IDS: Map<PollId, ()> = Map::new("participation_proposal_ids");

pub const PARTICIPATION_TOTAL_WEIGHT: Item<Uint128> = Item::new("participation_total_weight");

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
