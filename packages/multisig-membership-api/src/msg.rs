use crate::api::{ConfigResponse, SetMembersMsg, UpdateMembersMsg, UserWeight};
use cosmwasm_schema::{cw_serde, QueryResponses};
use membership_common_api::api::{
    MembersParams, MembersResponse, TotalWeightParams, TotalWeightResponse, UserWeightParams,
    UserWeightResponse, WeightChangeHookMsg,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_contract: String,
    pub initial_weights: Option<Vec<UserWeight>>,
    pub weight_change_hooks: Option<Vec<String>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateMembers(UpdateMembersMsg),
    SetMembers(SetMembersMsg),
    AddWeightChangeHook(WeightChangeHookMsg),
    RemoveWeightChangeHook(WeightChangeHookMsg),
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

#[cw_serde]
pub struct MigrateMsg {}
