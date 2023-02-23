use crate::cw20_distributions::{Cw20Distribution, CW20_DISTRIBUTIONS};
use crate::native_distributions::{NativeDistribution, NATIVE_DISTRIBUTIONS};
use crate::rewards::{calculate_cw20_user_reward, calculate_native_user_reward};
use crate::state::{CW20_GLOBAL_INDICES, ENTERPRISE_CONTRACT, NATIVE_GLOBAL_INDICES, TOTAL_WEIGHT};
use common::cw::Context;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, Decimal, Order, Response, StdResult, Uint128};
use funds_distributor_api::api::UpdateUserWeightsMsg;
use funds_distributor_api::error::DistributorError::Unauthorized;
use funds_distributor_api::error::DistributorResult;

pub fn update_user_weights(
    ctx: &mut Context,
    msg: UpdateUserWeightsMsg,
) -> DistributorResult<Response> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

    if ctx.info.sender != enterprise_contract {
        return Err(Unauthorized);
    }

    for old_user_weight in msg.old_user_weights {
        let user = ctx.deps.api.addr_validate(&old_user_weight.user)?;

        if old_user_weight.weight.is_zero() {
            // we may not have encountered this user, so we need to ensure their distribution
            // indices are set to current global indices
            initialize_user_indices(ctx, user.clone())?;
        } else {
            update_user_native_distributions(ctx, user.clone(), old_user_weight.weight)?;
            update_user_cw20_distributions(ctx, user, old_user_weight.weight)?;
        }
    }

    TOTAL_WEIGHT.save(ctx.deps.storage, &msg.new_total_weight)?;

    Ok(Response::new().add_attribute("action", "update_user_weights"))
}

fn initialize_user_indices(ctx: &mut Context, user: Addr) -> DistributorResult<()> {
    let native_global_indices = NATIVE_GLOBAL_INDICES
        .range(ctx.deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(String, Decimal)>>>()?;

    for (denom, global_index) in native_global_indices {
        NATIVE_DISTRIBUTIONS().update(
            ctx.deps.storage,
            (user.clone(), denom.clone()),
            |distribution| -> StdResult<NativeDistribution> {
                match distribution {
                    None => Ok(NativeDistribution {
                        user: user.clone(),
                        denom,
                        user_index: global_index,
                        pending_rewards: Uint128::zero(),
                    }),
                    Some(distribution) => Ok(distribution),
                }
            },
        )?;
    }

    let cw20_global_indices = CW20_GLOBAL_INDICES
        .range(ctx.deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(Addr, Decimal)>>>()?;

    for (asset, global_index) in cw20_global_indices {
        CW20_DISTRIBUTIONS().update(
            ctx.deps.storage,
            (user.clone(), asset.clone()),
            |distribution| -> StdResult<Cw20Distribution> {
                match distribution {
                    None => Ok(Cw20Distribution {
                        user: user.clone(),
                        cw20_asset: asset,
                        user_index: global_index,
                        pending_rewards: Uint128::zero(),
                    }),
                    Some(distribution) => Ok(distribution),
                }
            },
        )?;
    }

    Ok(())
}

fn update_user_native_distributions(
    ctx: &mut Context,
    user: Addr,
    old_user_weight: Uint128,
) -> DistributorResult<()> {
    let native_global_indices = NATIVE_GLOBAL_INDICES
        .range(ctx.deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(String, Decimal)>>>()?;

    for (denom, global_index) in native_global_indices {
        let distribution =
            NATIVE_DISTRIBUTIONS().may_load(ctx.deps.storage, (user.clone(), denom.clone()))?;

        let reward = calculate_native_user_reward(global_index, distribution, old_user_weight);

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
    let cw20_global_indices = CW20_GLOBAL_INDICES
        .range(ctx.deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<(Addr, Decimal)>>>()?;

    for (cw20_asset, global_index) in cw20_global_indices {
        let distribution =
            CW20_DISTRIBUTIONS().may_load(ctx.deps.storage, (user.clone(), cw20_asset.clone()))?;

        let reward = calculate_cw20_user_reward(global_index, distribution, old_user_weight);

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
