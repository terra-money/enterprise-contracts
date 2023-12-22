use crate::cw20_distributions::{Cw20Distribution, CW20_DISTRIBUTIONS};
use crate::eligibility::MINIMUM_ELIGIBLE_WEIGHT;
use crate::native_distributions::{NativeDistribution, NATIVE_DISTRIBUTIONS};
use crate::state::{ADMIN, CW20_GLOBAL_INDICES, EFFECTIVE_TOTAL_WEIGHT, NATIVE_GLOBAL_INDICES};
use crate::{cw20_distributions, native_distributions};
use common::cw::Context;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, Decimal, DepsMut, Response, StdResult, Uint128};
use cw20_distributions::update_user_cw20_distributions;
use cw_storage_plus::Map;
use funds_distributor_api::api::{UpdateUserWeightsMsg, UserWeight};
use funds_distributor_api::error::DistributorError::Unauthorized;
use funds_distributor_api::error::{DistributorError, DistributorResult};
use funds_distributor_api::response::execute_update_user_weights_response;
use native_distributions::update_user_native_distributions;
use DistributorError::DuplicateInitialWeight;

pub const USER_WEIGHTS: Map<Addr, Uint128> = Map::new("user_weights");

/// Effective user weights are their weights when taking into account minimum eligible weight
/// for rewards.
/// This weight will be the same as user's real weight if they're over the minimum eligible weight,
/// or 0 if they are under the minimum.
pub const EFFECTIVE_USER_WEIGHTS: Map<Addr, Uint128> = Map::new("effective_user_weights");

/// Saves any initial weights given to the users.
///
/// Should only be called when the contract is 'fresh'.
/// Do *NOT* call after there have already been reward distributions.
pub fn save_initial_weights(
    ctx: &mut Context,
    initial_weights: Vec<UserWeight>,
    minimum_eligible_weight: Uint128,
) -> DistributorResult<()> {
    let mut effective_total_weight = EFFECTIVE_TOTAL_WEIGHT
        .may_load(ctx.deps.storage)?
        .unwrap_or_default();

    for user_weight in initial_weights {
        let user = ctx.deps.api.addr_validate(&user_weight.user)?;

        if USER_WEIGHTS.has(ctx.deps.storage, user.clone())
            || EFFECTIVE_USER_WEIGHTS.has(ctx.deps.storage, user.clone())
        {
            return Err(DuplicateInitialWeight);
        }

        USER_WEIGHTS.save(ctx.deps.storage, user.clone(), &user_weight.weight)?;

        let effective_user_weight =
            calculate_effective_weight(user_weight.weight, minimum_eligible_weight);
        EFFECTIVE_USER_WEIGHTS.save(ctx.deps.storage, user, &effective_user_weight)?;

        effective_total_weight += effective_user_weight;
    }

    EFFECTIVE_TOTAL_WEIGHT.save(ctx.deps.storage, &effective_total_weight)?;

    Ok(())
}

/// Updates the users' weights to new ones.
/// Will calculate any accrued rewards since the last update to their rewards.
pub fn update_user_weights(
    ctx: &mut Context,
    msg: UpdateUserWeightsMsg,
) -> DistributorResult<Response> {
    let admin = ADMIN.load(ctx.deps.storage)?;

    if ctx.info.sender != admin {
        return Err(Unauthorized);
    }

    update_user_weights_checked(ctx.deps.branch(), msg)
}

pub fn update_user_weights_checked(
    mut deps: DepsMut,
    msg: UpdateUserWeightsMsg,
) -> DistributorResult<Response> {
    let mut effective_total_weight = EFFECTIVE_TOTAL_WEIGHT.load(deps.storage)?;

    let minimum_eligible_weight = MINIMUM_ELIGIBLE_WEIGHT.load(deps.storage)?;

    for user_weight_change in msg.new_user_weights {
        let user = deps.api.addr_validate(&user_weight_change.user)?;

        let old_user_effective_weight =
            EFFECTIVE_USER_WEIGHTS.may_load(deps.storage, user.clone())?;

        match old_user_effective_weight {
            None => {
                // we have not encountered this user, so we need to ensure their distribution
                // indices are set to current global indices
                initialize_user_indices(deps.branch(), user.clone())?;
            }
            Some(old_user_effective_weight) => {
                // the user already had their weight previously, so we use that weight
                // to calculate how many rewards for each asset they've accrued since we last
                // calculated their pending rewards
                update_user_native_distributions(
                    deps.branch(),
                    user.clone(),
                    old_user_effective_weight,
                )?;
                update_user_cw20_distributions(
                    deps.branch(),
                    user.clone(),
                    old_user_effective_weight,
                )?;
            }
        };

        USER_WEIGHTS.save(deps.storage, user.clone(), &user_weight_change.weight)?;

        let effective_user_weight =
            calculate_effective_weight(user_weight_change.weight, minimum_eligible_weight);
        EFFECTIVE_USER_WEIGHTS.save(deps.storage, user, &effective_user_weight)?;

        let old_user_effective_weight = old_user_effective_weight.unwrap_or_default();

        effective_total_weight =
            effective_total_weight - old_user_effective_weight + effective_user_weight;
    }

    EFFECTIVE_TOTAL_WEIGHT.save(deps.storage, &effective_total_weight)?;

    Ok(execute_update_user_weights_response())
}

/// Calculate user's effective rewards weight, given their actual weight and minimum weight for
/// rewards eligibility
fn calculate_effective_weight(weight: Uint128, minimum_eligible_weight: Uint128) -> Uint128 {
    if weight >= minimum_eligible_weight {
        weight
    } else {
        Uint128::zero()
    }
}

/// Called for users that we did not encounter previously.
///
/// Will initialize all their rewards for assets with existing distributions to 0, and set
/// their rewards indices to current global index for each asset.
fn initialize_user_indices(deps: DepsMut, user: Addr) -> DistributorResult<()> {
    let native_global_indices = NATIVE_GLOBAL_INDICES
        .range(deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<(String, Decimal)>>>()?;

    for (denom, global_index) in native_global_indices {
        NATIVE_DISTRIBUTIONS().update(
            deps.storage,
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
        .range(deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<(Addr, Decimal)>>>()?;

    for (asset, global_index) in cw20_global_indices {
        CW20_DISTRIBUTIONS().update(
            deps.storage,
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
