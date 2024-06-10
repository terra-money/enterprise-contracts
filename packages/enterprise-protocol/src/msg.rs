use crate::api::{
    ComponentContractsResponse, DaoInfoResponse, DaoMetadata, DaoType, ExecuteMsgsMsg,
    FinalizeInstantiationMsg, UpdateConfigMsg, UpdateMetadataMsg, UpgradeDaoMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Timestamp;
use enterprise_versioning_api::api::Version;

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_factory_contract: String,
    pub enterprise_versioning_contract: String,
    pub dao_metadata: DaoMetadata,
    pub dao_creation_date: Option<Timestamp>,
    pub dao_type: DaoType,
    pub dao_version: Version,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateMetadata(UpdateMetadataMsg),
    UpgradeDao(UpgradeDaoMsg),

    ExecuteMsgs(ExecuteMsgsMsg),

    UpdateConfig(UpdateConfigMsg),

    // called only right after instantiation
    FinalizeInstantiation(FinalizeInstantiationMsg),
}

#[cw_serde]
pub struct MigrateMsg {
    pub move_excess_membership_assets_to: Option<String>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DaoInfoResponse)]
    DaoInfo {},

    #[returns(ComponentContractsResponse)]
    ComponentContracts {},
}
