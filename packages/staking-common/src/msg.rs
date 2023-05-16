use cosmwasm_schema::cw_serde;
use cw_utils::Duration;

#[cw_serde]
pub struct InstantiateMsg {
    /// Admin of this contract. Almost all operations are limited to admin-only.
    pub admin: String,
    /// Asset (token, NFT, etc.) that is being staked in this contract.
    pub asset_contract: String,
    pub unlocking_period: Duration,
}
