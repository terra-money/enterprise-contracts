use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_utils::Expiration;

#[cw_serde]
pub struct UpdateAdminMsg {
    pub new_admin: String,
}

#[cw_serde]
pub struct UserWeightParams {
    pub user: String,
}

#[cw_serde]
pub struct TotalWeightParams {
    /// Denotes the moment at which we're interested in the total weight.
    /// Expiration::Never is used for current total weight.
    pub expiration: Expiration, // TODO: name this 'history_moment' or sth?
}

#[cw_serde]
pub struct MembersParams {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct UserWeightChange {
    pub user: String,
    pub old_weight: Uint128,
    pub new_weight: Uint128,
}

#[cw_serde]
pub struct WeightChangeHookMsg {
    pub hook_addr: String,
}

#[cw_serde]
pub struct WeightsChangedMsg {
    pub weight_changes: Vec<UserWeightChange>,
}

////// Responses

#[cw_serde]
pub struct AdminResponse {
    pub admin: Addr,
}

#[cw_serde]
pub struct UserWeightResponse {
    pub user: Addr,
    pub weight: Uint128,
}

#[cw_serde]
pub struct TotalWeightResponse {
    pub total_weight: Uint128,
}

#[cw_serde]
pub struct MembersResponse {
    pub members: Vec<UserWeightResponse>,
}
