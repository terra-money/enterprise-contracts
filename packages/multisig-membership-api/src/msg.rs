use crate::api::{
    TotalWeightParams, TotalWeightResponse, UpdateConfigMsg, UpdateMembersMsg, UserWeightParams,
    UserWeightResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateMembers(UpdateMembersMsg),
    UpdateConfig(UpdateConfigMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(UserWeightResponse)]
    UserWeight(UserWeightParams),
    #[returns(TotalWeightResponse)]
    TotalWeight(TotalWeightParams),
}

#[cw_serde]
pub struct MigrateMsg {}
