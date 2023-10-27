use crate::config::{Config, CONFIG};
use common::cw::Context;
use cosmwasm_std::Uint128;
use membership_common::enterprise_contract::set_enterprise_contract;
use membership_common::total_weight::{save_initial_total_weight_checkpoints, save_total_weight};
use membership_common::weight_change_hooks::save_initial_weight_change_hooks;
use nft_staking_api::error::NftStakingResult;
use nft_staking_api::msg::InstantiateMsg;

pub fn instantiate(ctx: &mut Context, msg: InstantiateMsg) -> NftStakingResult<()> {
    set_enterprise_contract(ctx.deps.branch(), msg.enterprise_contract)?;

    let nft_contract = ctx.deps.api.addr_validate(&msg.nft_contract)?;

    let config = Config {
        nft_contract,
        unlocking_period: msg.unlocking_period,
    };

    CONFIG.save(ctx.deps.storage, &config)?;

    save_initial_total_weight_checkpoints(
        ctx.deps.storage,
        msg.total_weight_by_height_checkpoints.unwrap_or_default(),
        msg.total_weight_by_seconds_checkpoints.unwrap_or_default(),
    )?;

    save_total_weight(ctx.deps.storage, &Uint128::zero(), &ctx.env.block)?;

    if let Some(weight_change_hooks) = msg.weight_change_hooks {
        save_initial_weight_change_hooks(ctx, weight_change_hooks)?;
    }

    Ok(())
}
