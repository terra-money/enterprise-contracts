use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Map;

pub const MEMBER_WEIGHTS: Map<Addr, Uint128> = Map::new("member_weights");
