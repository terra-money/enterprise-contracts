use cosmwasm_schema::cw_serde;
use cosmwasm_std::{BlockInfo, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use enterprise_governance_controller_api::api::{
    ProposalAction, ProposalDeposit, ProposalId, ProposalType,
};
use enterprise_governance_controller_api::error::GovernanceControllerError::NoSuchProposal;
use enterprise_governance_controller_api::error::GovernanceControllerResult;

pub const PROPOSAL_INFOS: Map<ProposalId, ProposalInfo> = Map::new("proposal_infos");

pub const TOTAL_DEPOSITS: Item<Uint128> = Item::new("total_proposal_deposits");

#[cw_serde]
pub struct ProposalInfo {
    pub proposal_type: ProposalType,
    pub executed_at: Option<BlockInfo>,
    pub proposal_deposit: Option<ProposalDeposit>,
    pub proposal_actions: Vec<ProposalAction>,
}

pub fn is_proposal_executed(
    store: &dyn Storage,
    proposal_id: ProposalId,
) -> GovernanceControllerResult<bool> {
    PROPOSAL_INFOS
        .may_load(store, proposal_id)?
        .map(|info| info.executed_at.is_some())
        .ok_or(NoSuchProposal)
}

pub fn set_proposal_executed(
    store: &mut dyn Storage,
    proposal_id: ProposalId,
    block: BlockInfo,
) -> GovernanceControllerResult<()> {
    PROPOSAL_INFOS.update(
        store,
        proposal_id,
        |info| -> GovernanceControllerResult<ProposalInfo> {
            info.map(|info| ProposalInfo {
                executed_at: Some(block),
                ..info
            })
            .ok_or(NoSuchProposal)
        },
    )?;

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
