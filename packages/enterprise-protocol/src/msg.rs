use crate::api::{
    ComponentContractsResponse, DaoInfoResponse, DaoMetadata, FinalizeInstantiationMsg,
    UpdateMetadataMsg, UpgradeDaoMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_factory_contract: String,
    pub enterprise_versioning_contract: String,
    pub dao_metadata: DaoMetadata,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateMetadata(UpdateMetadataMsg),
    UpgradeDao(UpgradeDaoMsg),

    // called by this contract itself
    FinalizeInstantiation(FinalizeInstantiationMsg),
    FinalizeMigration {},
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DaoInfoResponse)]
    DaoInfo {},
    #[returns(ComponentContractsResponse)]
    ComponentContracts {},
}
