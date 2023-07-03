use crate::config::{Config, CONFIG};
use common::cw::Context;
use cosmwasm_std::Uint128;
use membership_common::admin::ADMIN;
use membership_common::total_weight::save_total_weight;
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

    save_total_weight(ctx.deps.storage, &Uint128::zero(), &ctx.env.block)?;

    Ok(())
}
