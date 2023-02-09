use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const ENTERPRISE_CONTRACT: Item<Addr> = Item::new("enterprise_contract");
