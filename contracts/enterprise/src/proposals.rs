use cosmwasm_schema::cw_serde;
use cosmwasm_std::{BlockInfo, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use enterprise_protocol::api::{ProposalAction, ProposalDeposit, ProposalId};
use enterprise_protocol::error::DaoError::NoSuchProposal;
use enterprise_protocol::error::DaoResult;

// TODO: this is junk, unify somehow; causes a lot of duplication wherever we match on this value
#[cw_serde]
pub enum ProposalType {
    General,
    Council,
}

pub const PROPOSAL_INFOS: Map<ProposalId, ProposalInfo> = Map::new("proposal_infos");
pub const COUNCIL_PROPOSAL_INFOS: Map<ProposalId, ProposalInfo> =
    Map::new("council_proposal_infos");

// TODO: test usages of this, in relation to excluding deposits from treasury queries
pub const TOTAL_DEPOSITS: Item<Uint128> = Item::new("total_proposal_deposits");

#[cw_serde]
pub struct ProposalInfo {
    pub executed_at: Option<BlockInfo>,
    pub proposal_deposit: Option<ProposalDeposit>,
    pub proposal_actions: Vec<ProposalAction>,
}

pub fn is_proposal_executed(
    store: &dyn Storage,
    proposal_id: ProposalId,
    proposal_type: ProposalType,
) -> DaoResult<bool> {
    match proposal_type {
        ProposalType::General => PROPOSAL_INFOS
            .may_load(store, proposal_id)?
            .map(|info| info.executed_at.is_some()),
        ProposalType::Council => COUNCIL_PROPOSAL_INFOS
            .may_load(store, proposal_id)?
            .map(|info| info.executed_at.is_some()),
    }
    .ok_or(NoSuchProposal)
}

pub fn set_proposal_executed(
    store: &mut dyn Storage,
    proposal_id: ProposalId,
    block: BlockInfo,
    proposal_type: ProposalType,
) -> DaoResult<()> {
    match proposal_type {
        ProposalType::General => {
            PROPOSAL_INFOS.update(store, proposal_id, |info| -> DaoResult<ProposalInfo> {
                info.map(|info| ProposalInfo {
                    executed_at: Some(block),
                    ..info
                })
                .ok_or(NoSuchProposal)
            })?;
        }
        ProposalType::Council => {
            COUNCIL_PROPOSAL_INFOS.update(
                store,
                proposal_id,
                |info| -> DaoResult<ProposalInfo> {
                    info.map(|info| ProposalInfo {
                        executed_at: Some(block),
                        ..info
                    })
                    .ok_or(NoSuchProposal)
                },
            )?;
        }
    }

    Ok(())
}

pub fn get_proposal_actions(
    store: &dyn Storage,
    proposal_id: ProposalId,
    proposal_type: ProposalType,
) -> StdResult<Option<Vec<ProposalAction>>> {
    match proposal_type {
        ProposalType::General => PROPOSAL_INFOS
            .may_load(store, proposal_id)
            .map(|info_opt| info_opt.map(|info| info.proposal_actions)),
        ProposalType::Council => COUNCIL_PROPOSAL_INFOS
            .may_load(store, proposal_id)
            .map(|info_opt| info_opt.map(|info| info.proposal_actions)),
    }
}
