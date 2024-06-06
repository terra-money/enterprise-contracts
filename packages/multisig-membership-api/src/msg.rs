use crate::api::{ConfigResponse, SetMembersMsg, UpdateMembersMsg, UserWeight};
use cosmwasm_schema::{cw_serde, QueryResponses};
use membership_common_api::api::{MembersParams, MembersResponse, TotalWeightAboveParams, TotalWeightCheckpoint, TotalWeightParams, TotalWeightResponse, UserWeightParams, UserWeightResponse, WeightChangeHookMsg};

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_contract: String,
    pub initial_weights: Option<Vec<UserWeight>>,
    pub weight_change_hooks: Option<Vec<String>>,
    pub total_weight_by_height_checkpoints: Option<Vec<TotalWeightCheckpoint>>,
    pub total_weight_by_seconds_checkpoints: Option<Vec<TotalWeightCheckpoint>>,
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
    #[returns(TotalWeightResponse)]
    TotalWeightAbove(TotalWeightAboveParams),
    #[returns(MembersResponse)]
    Members(MembersParams),
}

#[cw_serde]
pub struct MigrateMsg {}
