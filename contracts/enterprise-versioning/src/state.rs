use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use enterprise_versioning_api::api::Version;

pub const ADMIN: Item<Addr> = Item::new("admin");

pub const VERSIONS: Map<u64, Version> = Map::new("versions");
