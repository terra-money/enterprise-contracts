use crate::cw20_distributions::{Cw20Distribution, CW20_DISTRIBUTIONS};
use crate::native_distributions::{NativeDistribution, NATIVE_DISTRIBUTIONS};
use crate::rewards::{calculate_cw20_user_reward, calculate_native_user_reward};
use crate::state::{CW20_GLOBAL_INDICES, ENTERPRISE_CONTRACT, NATIVE_GLOBAL_INDICES, TOTAL_STAKED};
use common::cw::Context;
use cosmwasm_std::{Addr, Decimal, Order, Response, StdResult, Uint128};
use funds_distributor_api::api::{UpdateTotalStakedMsg, UpdateUserStakeMsg};
use funds_distributor_api::error::DistributorError::Unauthorized;
use funds_distributor_api::error::DistributorResult;

pub fn update_total_staked(
    ctx: &mut Context,
    msg: UpdateTotalStakedMsg,
) -> DistributorResult<Response> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

    if ctx.info.sender != enterprise_contract {
        return Err(Unauthorized);
    }

    TOTAL_STAKED.save(ctx.deps.storage, &msg.new_total_staked)?;

    Ok(Response::new()
        .add_attribute("action", "update_total_staked")
        .add_attribute("new_total_staked", msg.new_total_staked.to_string()))
}

pub fn update_user_staked(
    ctx: &mut Context,
    msg: UpdateUserStakeMsg,
) -> DistributorResult<Response> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

    if ctx.info.sender != enterprise_contract {
        return Err(Unauthorized);
    }

    let user = ctx.deps.api.addr_validate(&msg.user)?;

    update_user_native_distributions(ctx, user.clone(), msg.old_user_stake)?;
    update_user_cw20_distributions(ctx, user.clone(), msg.old_user_stake)?;

    TOTAL_STAKED.save(ctx.deps.storage, &msg.new_total_staked)?;

    Ok(Response::new()
        .add_attribute("action", "update_user_staked")
        .add_attribute("user", user.to_string())
        .add_attribute("old_user_staked", msg.old_user_stake.to_string()))
}

fn update_user_native_distributions(
    ctx: &mut Context,
    user: Addr,
    old_user_stake: Uint128,
) -> DistributorResult<()> {
    let native_global_indices = NATIVE_GLOBAL_INDICES
        .range(ctx.deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(String, Decimal)>>>()?;

    for (denom, global_index) in native_global_indices {
        let native_distribution =
            NATIVE_DISTRIBUTIONS().may_load(ctx.deps.storage, (user.clone(), denom.clone()))?;

        let reward =
            calculate_native_user_reward(global_index, native_distribution, old_user_stake);

        NATIVE_DISTRIBUTIONS().save(
            ctx.deps.storage,
            (user.clone(), denom.clone()),
            &NativeDistribution {
                user: user.clone(),
                denom,
                user_index: global_index,
                pending_rewards: reward,
            },
        )?;
    }

    Ok(())
}

fn update_user_cw20_distributions(
    ctx: &mut Context,
    user: Addr,
    old_user_stake: Uint128,
) -> DistributorResult<()> {
    let cw20_global_indices = CW20_GLOBAL_INDICES
        .range(ctx.deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(Addr, Decimal)>>>()?;

    for (cw20_asset, global_index) in cw20_global_indices {
        let cw20_distribution =
            CW20_DISTRIBUTIONS().may_load(ctx.deps.storage, (user.clone(), cw20_asset.clone()))?;

        let reward = calculate_cw20_user_reward(global_index, cw20_distribution, old_user_stake);

        CW20_DISTRIBUTIONS().save(
            ctx.deps.storage,
            (user.clone(), cw20_asset.clone()),
            &Cw20Distribution {
                user: user.clone(),
                cw20_asset,
                user_index: global_index,
                pending_rewards: reward,
            },
        )?;
    }

    Ok(())
}
