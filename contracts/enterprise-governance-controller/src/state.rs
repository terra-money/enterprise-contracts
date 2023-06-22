use crate::proposals::ProposalInfo;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use enterprise_governance_controller_api::api::{DaoCouncilGovConfig, GovConfig, ProposalId};

#[cw_serde]
pub struct State {
    pub proposal_being_created: Option<ProposalInfo>,
    pub proposal_being_executed: Option<ProposalId>,
}

pub const STATE: Item<State> = Item::new("state");

pub const ENTERPRISE_CONTRACT: Item<Addr> = Item::new("enterprise_contract");

pub const GOV_CONFIG: Item<GovConfig> = Item::new("gov_config");
pub const DAO_COUNCIL_MEMBERSHIP_CONTRACT: Item<Addr> =
    Item::new("dao_council_membership_contract");
// TODO: change to DAO_COUNCIL_GOV_CONFIG
pub const DAO_COUNCIL: Item<Option<DaoCouncilGovConfig>> = Item::new("dao_council");
