use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError};
use cw_asset::AssetInfo;
use cw_storage_plus::{Item, Map};
use cw_utils::Duration;
use enterprise_factory_api::api::{Config, CreateDaoMsg};
use enterprise_protocol::api::DaoType;
use enterprise_protocol::error::DaoError::Std;
use enterprise_protocol::error::DaoResult;
use enterprise_versioning_api::api::VersionInfo;
use multisig_membership_api::api::UserWeight;

pub const CONFIG: Item<Config> = Item::new("config");

pub const DAO_ADDRESSES: Map<u64, Addr> = Map::new("dao_addresses");
pub const DAO_ID_COUNTER: Item<u64> = Item::new("dao_id_counter");

pub const ENTERPRISE_CODE_IDS: Map<u64, ()> = Map::new("enterprise_code_ids");

// TODO: add comments
pub const DAO_BEING_CREATED: Item<DaoBeingCreated> = Item::new("dao_being_created");

#[cw_serde]
pub struct DaoBeingCreated {
    // TODO: make those two non-optional, move this to a separate file, and make this one optional
    // TODO continued: introducing a 'require_dao_being_created' function
    pub create_dao_msg: Option<CreateDaoMsg>,
    pub version_info: Option<VersionInfo>,
    pub dao_asset: Option<AssetInfo>,
    pub dao_nft: Option<Addr>,
    pub enterprise_address: Option<Addr>,
    // TODO: make this explicitly initialized for every membership type?
    pub initial_weights: Option<Vec<UserWeight>>,
    pub dao_type: Option<DaoType>,
    pub unlocking_period: Option<Duration>,
    pub membership_address: Option<Addr>,
    pub funds_distributor_address: Option<Addr>,
    pub enterprise_governance_address: Option<Addr>,
    pub enterprise_governance_controller_address: Option<Addr>,
    pub enterprise_treasury_address: Option<Addr>,
}

impl DaoBeingCreated {
    // TODO: try cutting down on the verbosity here

    pub fn require_create_dao_msg(&self) -> DaoResult<CreateDaoMsg> {
        self.create_dao_msg.clone().ok_or(Std(StdError::generic_err(
            "invalid state - create DAO msg not present when expected",
        )))
    }

    pub fn require_version_info(&self) -> DaoResult<VersionInfo> {
        self.version_info.clone().ok_or(Std(StdError::generic_err(
            "invalid state - version info not present when expected",
        )))
    }

    pub fn require_enterprise_address(&self) -> DaoResult<Addr> {
        self.enterprise_address
            .clone()
            .ok_or(Std(StdError::generic_err(
                "invalid state - DAO address not present when expected",
            )))
    }

    pub fn require_dao_type(&self) -> DaoResult<DaoType> {
        self.dao_type.clone().ok_or(Std(StdError::generic_err(
            "invalid state - DAO type not present when expected",
        )))
    }

    pub fn require_unlocking_period(&self) -> DaoResult<Duration> {
        self.unlocking_period.ok_or(Std(StdError::generic_err(
            "invalid state - unlocking_period not present when expected",
        )))
    }

    pub fn require_membership_address(&self) -> DaoResult<Addr> {
        self.membership_address
            .clone()
            .ok_or(Std(StdError::generic_err(
                "invalid state - membership address not present when expected",
            )))
    }

    pub fn require_funds_distributor_address(&self) -> DaoResult<Addr> {
        self.funds_distributor_address
            .clone()
            .ok_or(Std(StdError::generic_err(
                "invalid state - funds distributor address not present when expected",
            )))
    }

    pub fn require_enterprise_governance_address(&self) -> DaoResult<Addr> {
        self.enterprise_governance_address
            .clone()
            .ok_or(Std(StdError::generic_err(
                "invalid state - Enterprise governance address not present when expected",
            )))
    }

    pub fn require_enterprise_governance_controller_address(&self) -> DaoResult<Addr> {
        self.enterprise_governance_controller_address
            .clone()
            .ok_or(Std(StdError::generic_err(
            "invalid state - Enterprise governance controller address not present when expected",
        )))
    }

    pub fn require_enterprise_treasury_address(&self) -> DaoResult<Addr> {
        self.enterprise_treasury_address
            .clone()
            .ok_or(Std(StdError::generic_err(
                "invalid state - Enterprise treasury address not present when expected",
            )))
    }
}
