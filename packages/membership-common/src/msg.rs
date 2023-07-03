use crate::api::{
    ConfigResponse, MembersParams, MembersResponse, TotalWeightParams, TotalWeightResponse,
    UpdateConfigMsg, UserWeightParams, UserWeightResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig(UpdateConfigMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(UserWeightResponse)]
    UserWeight(UserWeightParams),
    #[returns(TotalWeightResponse)]
    TotalWeight(TotalWeightParams),
    #[returns(MembersResponse)]
    Members(MembersParams),
}
