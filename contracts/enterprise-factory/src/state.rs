use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError};
use cw_storage_plus::{Item, Map};
use cw_utils::Duration;
use enterprise_factory_api::api::{Config, CreateDaoMembershipMsg};
use enterprise_protocol::api::DaoType;
use enterprise_protocol::error::DaoError::Std;
use enterprise_protocol::error::DaoResult;

pub const CONFIG: Item<Config> = Item::new("config");

pub const DAO_ADDRESSES: Map<u64, Addr> = Map::new("dao_addresses");
pub const DAO_ID_COUNTER: Item<u64> = Item::new("dao_id_counter");

pub const ENTERPRISE_CODE_IDS: Map<u64, ()> = Map::new("enterprise_code_ids");

// TODO: add comments
pub const DAO_BEING_CREATED: Item<DaoBeingCreated> = Item::new("dao_being_created");

#[cw_serde]
pub struct DaoBeingCreated {
    pub membership: Option<CreateDaoMembershipMsg>,
    pub enterprise_address: Option<Addr>,
    pub dao_type: Option<DaoType>,
    pub unlocking_period: Option<Duration>,
    pub membership_address: Option<Addr>,
}

impl DaoBeingCreated {
    pub fn require_enterprise_address(&self) -> DaoResult<Addr> {
        self.enterprise_address
            .clone()
            .ok_or(Std(StdError::generic_err(
                "invalid state - DAO address not present when expected",
            )))
    }

    pub fn require_membership(&self) -> DaoResult<CreateDaoMembershipMsg> {
        self.membership.clone().ok_or(Std(StdError::generic_err(
            "invalid state - DAO membership info not present when expected",
        )))
    }

    pub fn require_unlocking_period(&self) -> DaoResult<Duration> {
        self.unlocking_period.ok_or(Std(StdError::generic_err(
            "invalid state - unlocking_period info not present when expected",
        )))
    }
}
