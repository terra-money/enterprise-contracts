use crate::config::{Config, CONFIG};
use common::cw::Context;
use nft_staking_api::error::NftStakingResult;
use nft_staking_api::msg::InstantiateMsg;

pub fn instantiate(ctx: &mut Context, msg: InstantiateMsg) -> NftStakingResult<()> {
    let admin = ctx.deps.api.addr_validate(&msg.admin)?;
    let nft_contract = ctx.deps.api.addr_validate(&msg.nft_contract)?;

    let config = Config {
        admin,
        nft_contract,
    };

    CONFIG.save(ctx.deps.storage, &config)?;

    Ok(())
}
