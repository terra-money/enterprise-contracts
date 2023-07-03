use crate::config::{Config, CONFIG};
use common::cw::Context;
use membership_common::admin::ADMIN;
use token_staking_api::error::TokenStakingResult;
use token_staking_api::msg::InstantiateMsg;

pub fn instantiate(ctx: &mut Context, msg: InstantiateMsg) -> TokenStakingResult<()> {
    let admin = ctx.deps.api.addr_validate(&msg.admin)?;
    ADMIN.save(ctx.deps.storage, &admin)?;

    let token_contract = ctx.deps.api.addr_validate(&msg.token_contract)?;

    let config = Config {
        token_contract,
        unlocking_period: msg.unlocking_period,
    };

    CONFIG.save(ctx.deps.storage, &config)?;

    Ok(())
}
