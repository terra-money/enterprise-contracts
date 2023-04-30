use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128, Uint64};
use cw_utils::{Duration, Expiration};

// TODO: move to common and remove other declarations of this
#[cw_serde]
pub enum ReleaseAt {
    Timestamp(Timestamp),
    Height(Uint64),
}

#[cw_serde]
pub struct UnstakeMsg {
    pub user: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct ClaimMsg {
    pub user: String,
}

#[cw_serde]
pub struct UpdateConfigMsg {
    pub new_admin: Option<String>,
    pub new_token_contract: Option<String>,
    pub new_unlocking_period: Option<Duration>,
}

#[cw_serde]
pub struct UserTokenStakeParams {
    pub user: String,
}

#[cw_serde]
pub struct TotalStakedAmountParams {
    /// Denotes the moment at which we're interested in the total staked amount.
    /// Expiration::Never is used for current total staked.
    pub expiration: Expiration,
}

#[cw_serde]
pub struct ClaimsParams {
    pub user: String,
}

#[cw_serde]
pub struct TokenClaim {
    pub id: Uint64,
    pub user: Addr,
    pub amount: Uint128,
    pub release_at: ReleaseAt,
}

////// Responses

#[cw_serde]
pub struct UserTokenStakeResponse {
    pub user: Addr,
    pub staked_amount: Uint128,
}

#[cw_serde]
pub struct ClaimsResponse {
    pub claims: Vec<TokenClaim>,
}

#[cw_serde]
pub struct TotalStakedAmountResponse {
    pub total_staked_amount: Uint128,
}
