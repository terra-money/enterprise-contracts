use crate::api::{
    CrossChainDeploymentsParams, CrossChainDeploymentsResponse, CrossChainTreasuriesParams,
    CrossChainTreasuriesResponse, DeployCrossChainTreasuryMsg, ExecuteCrossChainTreasuryMsg,
    ExecuteMsgReplyCallbackMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    DeployCrossChainTreasury(DeployCrossChainTreasuryMsg),

    ExecuteCrossChainTreasury(ExecuteCrossChainTreasuryMsg),

    /// Callback from the ICS proxy contract.
    ExecuteMsgReplyCallback(ExecuteMsgReplyCallbackMsg),
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
