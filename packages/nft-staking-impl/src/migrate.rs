use crate::config::{Config, NftContractAddr, CONFIG};
use common::cw::Context;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use cw_utils::Duration;
use nft_staking_api::error::NftStakingResult;

#[cw_serde]
struct OldConfig {
    pub nft_contract: Addr,
    pub unlocking_period: Duration,
}

const OLD_CONFIG: Item<OldConfig> = Item::new("config");

pub fn migrate(ctx: &mut Context) -> NftStakingResult<()> {
    let old_config = OLD_CONFIG.load(ctx.deps.storage)?;

    CONFIG.save(
        ctx.deps.storage,
        &Config {
            nft_contract_addr: NftContractAddr::Cw721 {
                contract: old_config.nft_contract,
            },
            unlocking_period: old_config.unlocking_period,
        },
    )?;

    Ok(())
}
