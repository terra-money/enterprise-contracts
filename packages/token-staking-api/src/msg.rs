use crate::api::{
    ClaimMsg, ClaimsParams, ClaimsResponse, TokenConfigResponse, UnstakeMsg,
    UpdateUnlockingPeriodMsg, UserClaim, UserStake,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;
use cw_utils::Duration;
use membership_common_api::api::{
    AdminResponse, MembersParams, MembersResponse, TotalWeightParams, TotalWeightResponse,
    UpdateAdminMsg, UserWeightParams, UserWeightResponse, WeightChangeHookMsg,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub token_contract: String,
    pub unlocking_period: Duration,
}

#[cw_serde]
pub enum ExecuteMsg {
    Unstake(UnstakeMsg),
    Claim(ClaimMsg),
    UpdateAdmin(UpdateAdminMsg),
    UpdateUnlockingPeriod(UpdateUnlockingPeriodMsg),
    Receive(Cw20ReceiveMsg),
    AddWeightChangeHook(WeightChangeHookMsg),
    RemoveWeightChangeHook(WeightChangeHookMsg),
}

#[cw_serde]
pub enum Cw20HookMsg {
    Stake { user: String },
    InitializeStakers { stakers: Vec<UserStake> },
    AddClaims { claims: Vec<UserClaim> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AdminResponse)]
    Admin {},
    #[returns(TokenConfigResponse)]
    TokenConfig {},
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
