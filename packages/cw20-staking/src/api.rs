use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, SubMsg, Timestamp, Uint128, Uint64};
use cw_utils::Duration;

#[cw_serde]
pub struct Config {
    pub cw20_staking_asset: Addr,
    pub unstaking_period: Duration,
}

// TODO: move to common-staking
#[cw_serde]
pub enum ReleaseAt {
    Timestamp(Timestamp),
    Height(Uint64),
}

#[cw_serde]
pub struct UnstakingResponse {
    /// Staked amount left after unstaking.
    pub new_staked_amount: Uint128,
    /// Messages that will be non-empty if there's no unbonding period and unstaked amount
    /// can be claimed immediately.
    pub msgs: Vec<SubMsg>,
}

#[cw_serde]
pub struct Cw20Claim {
    pub id: u64,
    pub user: Addr,
    pub amount: Uint128,
    pub release_at: ReleaseAt,
}
