use cosmwasm_std::Addr;
use cw_storage_plus::Item;

// TODO: move to era repository
pub type EraId = u64;

pub const ADMIN: Item<Addr> = Item::new("admin");
pub const ENTERPRISE_CONTRACT: Item<Addr> = Item::new("enterprise_contract");
