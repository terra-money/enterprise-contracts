use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

pub const ENTERPRISE_CONTRACT: Item<Addr> = Item::new("enterprise_contract");

pub const TOTAL_STAKED: Item<Uint128> = Item::new("total_staked");

pub const NATIVE_GLOBAL_INDICES: Map<String, Decimal> = Map::new("native_global_indices");
pub const CW20_GLOBAL_INDICES: Map<Addr, Decimal> = Map::new("cw20_global_indices");
