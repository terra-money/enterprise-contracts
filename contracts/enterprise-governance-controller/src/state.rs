use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use enterprise_governance_controller_api::api::ProposalInfo;
use enterprise_governance_controller_api::api::{CouncilGovConfig, GovConfig, ProposalId};

#[cw_serde]
pub struct State {
    pub proposal_being_created: Option<ProposalInfo>,
    pub proposal_being_executed: Option<ProposalId>,
}

pub const STATE: Item<State> = Item::new("state");

pub const ENTERPRISE_CONTRACT: Item<Addr> = Item::new("enterprise_contract");

pub const GOV_CONFIG: Item<GovConfig> = Item::new("gov_config");

pub const COUNCIL_GOV_CONFIG: Item<Option<CouncilGovConfig>> = Item::new("council_gov_config");
