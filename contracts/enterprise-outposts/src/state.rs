use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const ENTERPRISE_CONTRACT: Item<Addr> = Item::new("enterprise_contract");

/// Proxies used on other chains to control treasuries.
/// Maps chain_id to proxy address (in its foreign-chain representation).
pub const CROSS_CHAIN_PROXIES: Map<String, String> = Map::new("cross_chain_proxies");

/// Treasuries used in addition to the main one.
/// Those are cross-chain, and this is the key part of our cross-chain design.
/// Maps chain_id to treasury address (in its foreign-chain representation).
pub const CROSS_CHAIN_TREASURIES: Map<String, String> = Map::new("cross_chain_treasuries");
