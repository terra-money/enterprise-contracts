use crate::config::{Config, CONFIG};
use common::cw::Context;
use nft_staking_api::error::NftStakingResult;
use staking_common::msg::InstantiateMsg;

pub fn instantiate(ctx: &mut Context, msg: InstantiateMsg) -> NftStakingResult<()> {
    let admin = ctx.deps.api.addr_validate(&msg.admin)?;
    let nft_contract = ctx.deps.api.addr_validate(&msg.asset_contract)?;

    let config = Config {
        admin,
        nft_contract,
        unlocking_period: msg.unlocking_period,
    };

    CONFIG.save(ctx.deps.storage, &config)?;

    Ok(())
}
