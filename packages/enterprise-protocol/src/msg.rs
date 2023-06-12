use crate::api::{
    ComponentContractsResponse, DaoInfoResponse, DaoMetadata, FinalizeInstantiationMsg,
    UpdateMetadataMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_factory_contract: String,
    pub dao_metadata: DaoMetadata,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Called by enterprise-factory contract to finalize instantiation (only once)
    FinalizeInstantiation(FinalizeInstantiationMsg),
    UpdateMetadata(UpdateMetadataMsg),
}

#[cw_serde]
pub struct MigrateMsg {
    pub treasury_code_id: u64,
    pub governance_controller_code_id: u64,
    pub token_membership_code_id: u64,
    pub nft_membership_code_id: u64,
    pub multisig_membership_code_id: u64,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DaoInfoResponse)]
    DaoInfo {},
    #[returns(ComponentContractsResponse)]
    ComponentContracts {},
}
