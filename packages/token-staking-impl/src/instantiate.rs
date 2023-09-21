use crate::config::{Config, CONFIG};
use common::cw::Context;
use cosmwasm_std::Uint128;
use membership_common::enterprise_contract::set_enterprise_contract;
use membership_common::total_weight::save_total_weight;
use membership_common::weight_change_hooks::save_initial_weight_change_hooks;
use token_staking_api::error::TokenStakingResult;
use token_staking_api::msg::InstantiateMsg;

pub fn instantiate(ctx: &mut Context, msg: InstantiateMsg) -> TokenStakingResult<()> {
    set_enterprise_contract(ctx.deps.branch(), msg.enterprise_contract)?;

    let token_contract = ctx.deps.api.addr_validate(&msg.token_contract)?;

    let config = Config {
        token_contract,
        unlocking_period: msg.unlocking_period,
    };

    CONFIG.save(ctx.deps.storage, &config)?;

    save_total_weight(ctx.deps.storage, &Uint128::zero(), &ctx.env.block)?;

    if let Some(weight_change_hooks) = msg.weight_change_hooks {
        save_initial_weight_change_hooks(ctx, weight_change_hooks)?;
    }

    Ok(())
}
