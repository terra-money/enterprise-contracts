use crate::api::{
    AdminResponse, MembersParams, MembersResponse, TotalWeightAboveParams, TotalWeightParams,
    TotalWeightResponse, UserWeightParams, UserWeightResponse, WeightChangeHookMsg,
    WeightsChangedMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub enum ExecuteMsg {
    AddWeightChangeHook(WeightChangeHookMsg),
    RemoveWeightChangeHook(WeightChangeHookMsg),
}

#[cw_serde]
pub enum WeightChangeHook {
    WeightsChanged(WeightsChangedMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AdminResponse)]
    Admin {},
    #[returns(UserWeightResponse)]
    UserWeight(UserWeightParams),
    #[returns(TotalWeightResponse)]
    TotalWeight(TotalWeightParams),
    #[returns(TotalWeightResponse)]
    TotalWeightAbove(TotalWeightAboveParams),
    #[returns(MembersResponse)]
    Members(MembersParams),
}
