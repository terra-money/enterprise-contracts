use cosmwasm_std::Addr;
use cw_asset::AssetInfo;
use cw_storage_plus::{Item, Map};
use enterprise_factory_api::api::Config;

pub const CONFIG: Item<Config> = Item::new("config");

pub const GLOBAL_ASSET_WHITELIST: Item<Vec<AssetInfo>> = Item::new("global_asset_whitelist");
pub const GLOBAL_NFT_WHITELIST: Item<Vec<Addr>> = Item::new("global_nft_whitelist");

pub const DAO_ADDRESSES: Map<u64, Addr> = Map::new("dao_addresses");
pub const DAO_ID_COUNTER: Item<u64> = Item::new("dao_id_counter");

pub const ENTERPRISE_CODE_IDS: Map<u64, ()> = Map::new("enterprise_code_ids");
