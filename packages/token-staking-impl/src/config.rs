use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use cw_utils::Duration;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub token_contract: Addr,
    pub unlocking_period: Duration,
}

pub const CONFIG: Item<Config> = Item::new("config");
