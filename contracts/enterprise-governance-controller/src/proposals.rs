use cosmwasm_std::{BlockInfo, StdResult, Storage};
use cw_storage_plus::Map;
use enterprise_governance_controller_api::api::{ProposalAction, ProposalId, ProposalInfo};
use enterprise_governance_controller_api::error::GovernanceControllerError::NoSuchProposal;
use enterprise_governance_controller_api::error::GovernanceControllerResult;

pub const PROPOSAL_INFOS: Map<ProposalId, ProposalInfo> = Map::new("proposal_infos");

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
