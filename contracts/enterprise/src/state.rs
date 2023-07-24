use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};
use enterprise_protocol::api::{DaoMetadata, DaoType};
use enterprise_versioning_api::api::Version;

pub const DAO_METADATA_KEY: &str = "dao_metadata";

pub const DAO_CREATION_DATE: Item<Timestamp> = Item::new("dao_creation_date");

// Address of contract which is used to calculate DAO membership
pub const DAO_MEMBERSHIP_CONTRACT: Item<Addr> = Item::new("dao_membership_contract");

#[cw_serde]
pub struct ComponentContracts {
    pub enterprise_governance_contract: Addr,
    pub enterprise_governance_controller_contract: Addr,
    /// This is the main treasury contract, that is used by default.
    pub enterprise_treasury_contract: Addr,
    pub funds_distributor_contract: Addr,
    pub membership_contract: Addr,
    pub council_membership_contract: Addr,
    pub attestation_contract: Option<Addr>,
}

pub const COMPONENT_CONTRACTS: Item<ComponentContracts> = Item::new("component_contracts");

/// Treasuries used in addition to the main one.
/// Those are cross-chain, and this is the key part of our cross-chain design.
pub const CROSS_CHAIN_TREASURIES: Map<Addr, ()> = Map::new("cross_chain_treasuries");

pub const ENTERPRISE_FACTORY_CONTRACT: Item<Addr> = Item::new("enterprise_factory_contract");
pub const ENTERPRISE_VERSIONING_CONTRACT: Item<Addr> = Item::new("enterprise_versioning_contract");
pub const IS_INSTANTIATION_FINALIZED: Item<bool> = Item::new("is_creation_finalized");

pub const DAO_TYPE: Item<DaoType> = Item::new("dao_type");
pub const DAO_VERSION: Item<Version> = Item::new("dao_version");
pub const DAO_METADATA: Item<DaoMetadata> = Item::new(DAO_METADATA_KEY);
