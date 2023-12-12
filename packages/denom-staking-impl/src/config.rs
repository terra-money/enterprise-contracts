use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;
use cw_utils::Duration;

#[cw_serde]
pub struct Config {
    pub denom: String,
    pub unlocking_period: Duration,
}

pub const CONFIG: Item<Config> = Item::new("config");
