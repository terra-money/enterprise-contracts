use crate::api::{
    ClaimMsg, ClaimsParams, ClaimsResponse, TokenConfigResponse, UnstakeMsg,
    UpdateUnlockingPeriodMsg, UserClaim, UserStake,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;
use cw_utils::Duration;
use membership_common_api::api::{MembersParams, MembersResponse, TotalWeightAboveParams, TotalWeightCheckpoint, TotalWeightParams, TotalWeightResponse, UserWeightParams, UserWeightResponse, WeightChangeHookMsg};

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_contract: String,
    pub token_contract: String,
    pub unlocking_period: Duration,
    pub weight_change_hooks: Option<Vec<String>>,
    pub total_weight_by_height_checkpoints: Option<Vec<TotalWeightCheckpoint>>,
    pub total_weight_by_seconds_checkpoints: Option<Vec<TotalWeightCheckpoint>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Unstake(UnstakeMsg),
    Claim(ClaimMsg),
    UpdateUnlockingPeriod(UpdateUnlockingPeriodMsg),
    Receive(Cw20ReceiveMsg),
    AddWeightChangeHook(WeightChangeHookMsg),
    RemoveWeightChangeHook(WeightChangeHookMsg),
}

#[cw_serde]
pub enum Cw20HookMsg {
    Stake { user: String },
    AddStakes { stakers: Vec<UserStake> },
    AddClaims { claims: Vec<UserClaim> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(TokenConfigResponse)]
    TokenConfig {},
    #[returns(UserWeightResponse)]
    UserWeight(UserWeightParams),
    #[returns(TotalWeightResponse)]
    TotalWeight(TotalWeightParams),
    #[returns(TotalWeightResponse)]
    TotalWeightAbove(TotalWeightAboveParams),
    #[returns(ClaimsResponse)]
    Claims(ClaimsParams),
    #[returns(ClaimsResponse)]
    ReleasableClaims(ClaimsParams),
    #[returns(MembersResponse)]
    Members(MembersParams),
}

#[cw_serde]
pub struct MigrateMsg {
    pub move_excess_membership_assets_to: Option<String>,
}
