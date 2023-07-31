use common::cw::ReleaseAt;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint64};
use cw_utils::Duration;

#[cw_serde]
pub struct UserStake {
    pub user: String,
    pub staked_amount: Uint128,
}

#[cw_serde]
pub struct UserClaim {
    pub user: String,
    pub claim_amount: Uint128,
    pub release_at: ReleaseAt,
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
pub struct UpdateUnlockingPeriodMsg {
    pub new_unlocking_period: Option<Duration>,
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
pub struct ClaimsResponse {
    pub claims: Vec<TokenClaim>,
}

#[cw_serde]
pub struct TokenConfigResponse {
    pub enterprise_contract: Addr,
    pub token_contract: Addr,
    pub unlocking_period: Duration,
}
