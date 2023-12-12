use crate::api::{
    ClaimMsg, ClaimsParams, ClaimsResponse, DenomConfigResponse, UnstakeMsg,
    UpdateUnlockingPeriodMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_utils::Duration;
use membership_common_api::api::{
    MembersParams, MembersResponse, TotalWeightParams, TotalWeightResponse, UserWeightParams,
    UserWeightResponse, WeightChangeHookMsg,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_contract: String,
    pub denom: String,
    pub unlocking_period: Duration,
    pub weight_change_hooks: Option<Vec<String>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Stake { user: Option<String> },
    Unstake(UnstakeMsg),
    Claim(ClaimMsg),
    UpdateUnlockingPeriod(UpdateUnlockingPeriodMsg),
    AddWeightChangeHook(WeightChangeHookMsg),
    RemoveWeightChangeHook(WeightChangeHookMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
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
