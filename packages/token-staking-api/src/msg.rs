use crate::api::{
    ClaimMsg, ClaimsParams, ClaimsResponse, ReleaseAt, TotalStakedAmountParams,
    TotalStakedAmountResponse, UnstakeMsg, UpdateConfigMsg, UserStake, UserTokenStakeParams,
    UserTokenStakeResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;
use cw_utils::Duration;

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
    AddClaim { user: String, release_at: ReleaseAt },
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
