use crate::cw20_distributions::{Cw20Distribution, CW20_DISTRIBUTIONS};
use crate::native_distributions::{NativeDistribution, NATIVE_DISTRIBUTIONS};
use crate::rewards::{calculate_cw20_user_reward, calculate_native_user_reward};
use crate::state::{CW20_GLOBAL_INDICES, ENTERPRISE_CONTRACT, NATIVE_GLOBAL_INDICES, TOTAL_WEIGHT};
use common::cw::Context;
use cosmwasm_std::{Addr, Order, Response, StdResult, Uint128};
use funds_distributor_api::api::UpdateUserWeightsMsg;
use funds_distributor_api::error::DistributorError::Unauthorized;
use funds_distributor_api::error::DistributorResult;
use itertools::Itertools;

pub fn update_user_weights(
    ctx: &mut Context,
    msg: UpdateUserWeightsMsg,
) -> DistributorResult<Response> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

    if ctx.info.sender != enterprise_contract {
        return Err(Unauthorized);
    }

    for old_user_weight in msg.old_user_weights {
        if old_user_weight.weight.is_zero() {
            // we can skip this user, they are not getting any rewards whatsoever
            continue;
        }

        let user = ctx.deps.api.addr_validate(&old_user_weight.user)?;

        update_user_native_distributions(ctx, user.clone(), old_user_weight.weight)?;
        update_user_cw20_distributions(ctx, user, old_user_weight.weight)?;
    }

    TOTAL_WEIGHT.save(ctx.deps.storage, &msg.new_total_weight)?;

    Ok(Response::new().add_attribute("action", "update_user_weights"))
}

fn update_user_native_distributions(
    ctx: &mut Context,
    user: Addr,
    old_user_weight: Uint128,
) -> DistributorResult<()> {
    let user_native_distributions = NATIVE_DISTRIBUTIONS()
        .idx
        .user
        .prefix(user.clone())
        .range(ctx.deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<((Addr, String), NativeDistribution)>>>()?
        .into_iter()
        .map(|(_, distribution)| distribution)
        .collect_vec();

    for distribution in user_native_distributions {
        let denom = distribution.denom.clone();
        let global_index = NATIVE_GLOBAL_INDICES
            .may_load(ctx.deps.storage, denom.clone())?
            .unwrap_or_default();

        let reward =
            calculate_native_user_reward(global_index, Some(distribution), old_user_weight);

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
    old_user_weight: Uint128,
) -> DistributorResult<()> {
    let user_cw20_distributions = CW20_DISTRIBUTIONS()
        .idx
        .user
        .prefix(user.clone())
        .range(ctx.deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<((Addr, Addr), Cw20Distribution)>>>()?
        .into_iter()
        .map(|(_, distribution)| distribution)
        .collect_vec();

    for distribution in user_cw20_distributions {
        let cw20_asset = distribution.cw20_asset.clone();
        let global_index = CW20_GLOBAL_INDICES
            .may_load(ctx.deps.storage, cw20_asset.clone())?
            .unwrap_or_default();

        let reward = calculate_cw20_user_reward(global_index, Some(distribution), old_user_weight);

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
