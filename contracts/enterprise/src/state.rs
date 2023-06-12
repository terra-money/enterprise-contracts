use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint64};
use cw_storage_plus::Item;
use enterprise_protocol::api::{DaoMetadata, DaoType};

pub const DAO_METADATA_KEY: &str = "dao_metadata";

pub const DAO_CREATION_DATE: Item<Timestamp> = Item::new("dao_creation_date");

// Address of contract which is used to calculate DAO membership
pub const DAO_MEMBERSHIP_CONTRACT: Item<Addr> = Item::new("dao_membership_contract");

#[cw_serde]
pub struct ComponentContracts {
    pub enterprise_governance_contract: Addr,
    pub enterprise_governance_controller_contract: Addr,
    pub enterprise_treasury_contract: Addr,
    pub funds_distributor_contract: Addr,
    pub membership_contract: Addr,
}

pub const COMPONENT_CONTRACTS: Item<ComponentContracts> = Item::new("component_contracts");

pub const ENTERPRISE_FACTORY_CONTRACT: Item<Addr> = Item::new("enterprise_factory_contract");
pub const IS_INSTANTIATION_FINALIZED: Item<bool> = Item::new("is_creation_finalized");

pub const DAO_TYPE: Item<DaoType> = Item::new("dao_type");
pub const DAO_CODE_VERSION: Item<Uint64> = Item::new("dao_code_version");
pub const DAO_METADATA: Item<DaoMetadata> = Item::new(DAO_METADATA_KEY);
