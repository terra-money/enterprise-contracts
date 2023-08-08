use crate::api::{
    ClaimMsg, ClaimsParams, ClaimsResponse, DenomConfigResponse, UnstakeMsg,
    UpdateUnlockingPeriodMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_utils::Duration;
use membership_common_api::api::{
    AdminResponse, MembersParams, MembersResponse, TotalWeightParams, TotalWeightResponse,
    UpdateAdminMsg, UserWeightParams, UserWeightResponse, WeightChangeHookMsg,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub denom: String,
    pub unlocking_period: Duration,
}

#[cw_serde]
pub enum ExecuteMsg {
    Stake {},
    Unstake(UnstakeMsg),
    Claim(ClaimMsg),
    UpdateAdmin(UpdateAdminMsg),
    UpdateUnlockingPeriod(UpdateUnlockingPeriodMsg),
    AddWeightChangeHook(WeightChangeHookMsg),
    RemoveWeightChangeHook(WeightChangeHookMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AdminResponse)]
    Admin {},
    #[returns(DenomConfigResponse)]
    DenomConfig {},
    #[returns(UserWeightResponse)]
    UserWeight(UserWeightParams),
    #[returns(TotalWeightResponse)]
    TotalWeight(TotalWeightParams),
    #[returns(ClaimsResponse)]
    Claims(ClaimsParams),
    #[returns(ClaimsResponse)]
    ReleasableClaims(ClaimsParams),
    #[returns(MembersResponse)]
    Members(MembersParams),
}

#[cw_serde]
pub struct MigrateMsg {}
