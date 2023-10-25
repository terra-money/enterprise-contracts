use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const ENTERPRISE_FACADE_V1: Item<Addr> = Item::new("facade_v1");
pub const ENTERPRISE_FACADE_V2: Item<Addr> = Item::new("facade_v2");
