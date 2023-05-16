use crate::api::{
    ClaimsResponse, UnstakeMsg, UserClaim, UserStake, UserTokenStakeParams, UserTokenStakeResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;
use staking_common::api::{
    ClaimMsg, ClaimsParams, TotalStakedAmountParams, TotalStakedAmountResponse, UpdateConfigMsg,
};

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
    #[returns(UserTokenStakeResponse)]
    UserStake(UserTokenStakeParams),
    #[returns(TotalStakedAmountResponse)]
    TotalStakedAmount(TotalStakedAmountParams),
    #[returns(ClaimsResponse)]
    Claims(ClaimsParams),
    #[returns(ClaimsResponse)]
    ReleasableClaims(ClaimsParams),
}

#[cw_serde]
pub struct MigrateMsg {}
