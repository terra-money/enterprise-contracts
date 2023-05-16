use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cw_utils::{Duration, Expiration};

#[cw_serde]
pub struct ClaimMsg {
    pub user: String,
}

#[cw_serde]
pub struct UpdateConfigMsg {
    pub new_admin: Option<String>,
    pub new_asset_contract: Option<String>,
    pub new_unlocking_period: Option<Duration>,
}

#[cw_serde]
pub struct UserTotalStakeParams {
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

////// Responses

#[cw_serde]
pub struct TotalStakedAmountResponse {
    pub total_staked_amount: Uint128,
}
