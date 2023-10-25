use crate::api::{
    AddCrossChainProxyMsg, AddCrossChainTreasuryMsg, CrossChainDeploymentsParams,
    CrossChainDeploymentsResponse, CrossChainTreasuriesParams, CrossChainTreasuriesResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddCrossChainProxy(AddCrossChainProxyMsg),
    AddCrossChainTreasury(AddCrossChainTreasuryMsg),
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CrossChainTreasuriesResponse)]
    CrossChainTreasuries(CrossChainTreasuriesParams),

    #[returns(CrossChainDeploymentsResponse)]
    CrossChainDeployments(CrossChainDeploymentsParams),
}
