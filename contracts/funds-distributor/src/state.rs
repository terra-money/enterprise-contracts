use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

pub const ADMIN: Item<Addr> = Item::new("admin");
pub const ENTERPRISE_CONTRACT: Item<Addr> = Item::new("enterprise_contract");

/// Total weight of all users eligible for rewards.
pub const EFFECTIVE_TOTAL_WEIGHT: Item<Uint128> = Item::new("total_weight");

/// Tracks global index for native denomination rewards.
/// Global index is simply a decimal number representing the amount of currency rewards paid
/// for a unit of user weight, since the beginning of time.
pub const NATIVE_GLOBAL_INDICES: Map<String, Decimal> = Map::new("native_global_indices");

/// Tracks global index for CW20 token rewards.
/// Global index is simply a decimal number representing the amount of currency rewards paid
/// for a unit of user weight, since the beginning of time.
pub const CW20_GLOBAL_INDICES: Map<Addr, Decimal> = Map::new("cw20_global_indices");

/// Tracks global index for native denomination rewards for participation
pub const PARTICIPATION_NATIVE_GLOBAL_INDICES: Map<String, Decimal> = Map::new("participation_native_global_indices");

/// Tracks global index for CW20 token rewards for participation.
pub const PARTICIPATION_CW20_GLOBAL_INDICES: Map<Addr, Decimal> = Map::new("participation_cw20_global_indices");
