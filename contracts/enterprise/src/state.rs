use crate::proposals::ProposalInfo;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint64};
use cw_storage_plus::{Item, Map};
use enterprise_protocol::api::{DaoCouncil, DaoGovConfig, DaoMetadata, DaoType, ProposalId};

#[cw_serde]
pub struct State {
    pub proposal_being_created: Option<ProposalInfo>,
    pub proposal_being_executed: Option<ProposalId>,
}

pub const STATE: Item<State> = Item::new("state");

pub const DAO_METADATA_KEY: &str = "dao_metadata";

pub const DAO_CREATION_DATE: Item<Timestamp> = Item::new("dao_creation_date");

// TODO: try to unify those below into a single storage structure

/// Address of contract which is used to calculate DAO membership
pub const DAO_MEMBERSHIP_CONTRACT: Item<Addr> = Item::new("dao_membership_contract");

pub const ENTERPRISE_FACTORY_CONTRACT: Item<Addr> = Item::new("enterprise_factory_contract");
pub const ENTERPRISE_GOVERNANCE_CONTRACT: Item<Addr> = Item::new("enterprise_governance_contract");
pub const FUNDS_DISTRIBUTOR_CONTRACT: Item<Addr> = Item::new("funds_distributor_contract");

pub const DAO_TYPE: Item<DaoType> = Item::new("dao_type");
pub const DAO_CODE_VERSION: Item<Uint64> = Item::new("dao_code_version");
pub const DAO_METADATA: Item<DaoMetadata> = Item::new(DAO_METADATA_KEY);
pub const DAO_GOV_CONFIG: Item<DaoGovConfig> = Item::new("dao_gov_config");
pub const DAO_COUNCIL: Item<Option<DaoCouncil>> = Item::new("dao_council");

pub const NFT_WHITELIST: Map<Addr, ()> = Map::new("nft_whitelist");

/// Address of the staking contract.
/// Only token and NFT DAOs have this.
pub const STAKING_CONTRACT: Item<Addr> = Item::new("staking_contract");
