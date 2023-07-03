use crate::api::{
    ClaimMsg, ClaimsParams, ClaimsResponse, ConfigResponse, UnstakeMsg, UpdateConfigMsg, UserClaim,
    UserStake,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;
use cw_utils::Duration;
use membership_common::api::{
    MembersParams, MembersResponse, TotalWeightParams, TotalWeightResponse, UserWeightParams,
    UserWeightResponse,
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
    UpdateConfig(UpdateConfigMsg),
    Receive(Cw20ReceiveMsg),
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
    #[returns(ConfigResponse)]
    Config {},
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
