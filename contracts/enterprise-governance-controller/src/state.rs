use crate::proposals::ProposalInfo;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use enterprise_governance_controller_api::api::{DaoCouncil, GovConfig, ProposalId};

#[cw_serde]
pub struct State {
    pub proposal_being_created: Option<ProposalInfo>,
    pub proposal_being_executed: Option<ProposalId>,
}

pub const STATE: Item<State> = Item::new("state");

pub const ENTERPRISE_CONTRACT: Item<Addr> = Item::new("enterprise_contract");

pub const GOV_CONFIG: Item<GovConfig> = Item::new("gov_config");
// TODO: remove, replace by a singular membership contract
pub const DAO_COUNCIL: Item<Option<DaoCouncil>> = Item::new("dao_council");
