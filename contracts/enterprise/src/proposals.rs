use cosmwasm_schema::cw_serde;
use cosmwasm_std::{BlockInfo, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use enterprise_protocol::api::{ProposalAction, ProposalDeposit, ProposalId};
use enterprise_protocol::error::{DaoError, DaoResult};

pub const PROPOSAL_INFOS: Map<ProposalId, ProposalInfo> = Map::new("proposal_infos");

// TODO: test usages of this, in relation to excluding deposits from treasury queries
pub const TOTAL_DEPOSITS: Item<Uint128> = Item::new("total_proposal_deposits");

#[cw_serde]
pub struct ProposalInfo {
    pub executed_at: Option<BlockInfo>,
    pub proposal_deposit: Option<ProposalDeposit>,
    pub proposal_actions: Vec<ProposalAction>,
}

pub fn is_proposal_executed(store: &dyn Storage, proposal_id: ProposalId) -> DaoResult<bool> {
    PROPOSAL_INFOS
        .may_load(store, proposal_id)?
        .map(|info| info.executed_at.is_some())
        .ok_or(DaoError::NoSuchProposal)
}

pub fn set_proposal_executed(
    store: &mut dyn Storage,
    proposal_id: ProposalId,
    block: BlockInfo,
) -> DaoResult<()> {
    PROPOSAL_INFOS.update(store, proposal_id, |info| -> DaoResult<ProposalInfo> {
        info.map(|info| ProposalInfo {
            executed_at: Some(block),
            ..info
        })
        .ok_or(DaoError::NoSuchProposal)
    })?;
    Ok(())
}

pub fn get_proposal_actions(
    store: &dyn Storage,
    proposal_id: ProposalId,
) -> StdResult<Option<Vec<ProposalAction>>> {
    PROPOSAL_INFOS
        .may_load(store, proposal_id)
        .map(|info_opt| info_opt.map(|info| info.proposal_actions))
}
