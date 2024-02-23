use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use cw_utils::Duration;

#[cw_serde]
pub struct Config {
    pub nft_contract_addr: NftContractAddr,
    pub unlocking_period: Duration,
}

#[cw_serde]
pub enum NftContractAddr {
    Cw721 { contract: Addr },
    Ics721 { contract: Addr, class_id: String },
}

pub const CONFIG: Item<Config> = Item::new("config_v1_2_0");
