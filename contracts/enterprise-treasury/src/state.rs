use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const NFT_WHITELIST: Map<Addr, ()> = Map::new("nft_whitelist");

#[cw_serde]
pub struct Config {
    pub admin: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ENTERPRISE_CONTRACT: Item<Addr> = Item::new("enterprise_contract");
