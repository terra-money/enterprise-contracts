use crate::api::Config;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

// TODO: sort out this junk: move to separate files maybe, add helper functions

pub const CONFIG: Item<Config> = Item::new("cw20_staking:config");

pub const STAKES: Map<Addr, Uint128> = Map::new("cw20_staking:stakes");
pub const TOTAL_STAKED: Item<Uint128> = Item::new("cw20_staking:total_staked");
