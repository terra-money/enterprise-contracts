use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const ENTERPRISE_VERSIONING: Item<Addr> = Item::new("enterprise_versioning");
