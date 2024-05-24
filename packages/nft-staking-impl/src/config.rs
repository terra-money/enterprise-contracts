use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use cw_utils::Duration;
use nft_staking_api::error::NftStakingError::Ics721StillNotTransferred;
use nft_staking_api::error::NftStakingResult;

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

impl Config {
    pub fn require_cw721_addr(&self) -> NftStakingResult<Addr> {
        // extract the NFT contract, or fail if this is still in the ICS721-not-transferred stage
        match &self.nft_contract_addr {
            NftContractAddr::Cw721 { contract } => Ok(contract.clone()),
            NftContractAddr::Ics721 { .. } => Err(Ics721StillNotTransferred),
        }
    }
}
