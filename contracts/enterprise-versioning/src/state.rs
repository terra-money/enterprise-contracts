use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use enterprise_versioning_api::api::VersionInfo;

pub const ADMIN: Item<Addr> = Item::new("admin");

pub const VERSIONS: Map<(u64, u64, u64), VersionInfo> = Map::new("versions");
