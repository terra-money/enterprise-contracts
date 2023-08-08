use crate::config::{Config, CONFIG};
use common::cw::Context;
use cosmwasm_std::Uint128;
use denom_staking_api::error::DenomStakingResult;
use denom_staking_api::msg::InstantiateMsg;
use membership_common::admin::ADMIN;
use membership_common::total_weight::save_total_weight;

pub fn instantiate(ctx: &mut Context, msg: InstantiateMsg) -> DenomStakingResult<()> {
    let admin = ctx.deps.api.addr_validate(&msg.admin)?;
    ADMIN.save(ctx.deps.storage, &admin)?;

    let config = Config {
        denom: msg.denom,
        unlocking_period: msg.unlocking_period,
    };

    CONFIG.save(ctx.deps.storage, &config)?;

    save_total_weight(ctx.deps.storage, &Uint128::zero(), &ctx.env.block)?;

    Ok(())
}
