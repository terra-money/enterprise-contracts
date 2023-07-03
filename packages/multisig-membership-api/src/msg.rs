use crate::api::{SetMembersMsg, UpdateMembersMsg, UserWeight};
use cosmwasm_schema::{cw_serde, QueryResponses};
use membership_common_api::api::{
    AdminResponse, MembersParams, MembersResponse, TotalWeightParams, TotalWeightResponse,
    UpdateAdminMsg, UserWeightParams, UserWeightResponse,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub initial_weights: Option<Vec<UserWeight>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateMembers(UpdateMembersMsg),
    SetMembers(SetMembersMsg),
    UpdateAdmin(UpdateAdminMsg),
    // TODO: uncomment
    // AddWeightChangeHook(WeightChangeHookMsg),
    // RemoveWeightChangeHook(WeightChangeHookMsg),
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
    #[returns(MembersResponse)]
    Members(MembersParams),
}

#[cw_serde]
pub struct MigrateMsg {}
