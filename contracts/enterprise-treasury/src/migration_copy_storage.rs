use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Map;

/// User weights that have already been transferred to membership contract,
/// but are being kept while the migration is still ongoing.
pub const MIGRATED_USER_WEIGHTS: Map<Addr, Uint128> = Map::new("migrated_user_weights");
