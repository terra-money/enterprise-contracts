use crate::repository::asset_repository::{
    asset_distribution_repository, AssetDistributionRepository,
};
use crate::repository::user_distribution_repository::{
    user_distribution_repository_mut, UserDistributionRepositoryMut,
};
use crate::repository::weights_repository::{
    weights_repository, weights_repository_mut, WeightsRepository, WeightsRepositoryMut,
};
use crate::state::{ADMIN, EFFECTIVE_TOTAL_WEIGHT};
use common::cw::Context;
use cosmwasm_std::{Addr, DepsMut, Response, Uint128};
use cw_storage_plus::Map;
use funds_distributor_api::api::{UpdateUserWeightsMsg, UserWeight};
use funds_distributor_api::error::DistributorError::Unauthorized;
use funds_distributor_api::error::{DistributorError, DistributorResult};
use funds_distributor_api::response::execute_update_user_weights_response;
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

    update_user_weights_checked(ctx.deps.branch(), msg)?;

    Ok(execute_update_user_weights_response())
}

fn update_user_weights_checked(
    mut deps: DepsMut,
    msg: UpdateUserWeightsMsg,
) -> DistributorResult<()> {
    let mut total_weight = weights_repository(deps.as_ref()).get_total_weight()?;

    for user_weight_change in msg.new_user_weights {
        let user = deps.api.addr_validate(&user_weight_change.user)?;

        let old_user_weight = weights_repository(deps.as_ref()).get_user_weight(user.clone())?;

        match old_user_weight {
            None => {
                // we have not encountered this user, so we need to ensure their distribution
                // indices are set to current global indices
                initialize_user_indices(deps.branch(), user.clone())?;
            }
            Some(old_user_weight) => {
                // the user already had their weight previously, so we use that weight
                // to calculate how many rewards for each asset they've accrued since we last
                // calculated their pending rewards
                update_user_indices(deps.branch(), user.clone(), old_user_weight)?;
            }
        };

        let new_user_weight = weights_repository_mut(deps.branch())
            .set_user_weight(user.clone(), user_weight_change.weight)?;

        let old_user_weight = old_user_weight.unwrap_or_default();

        total_weight = total_weight - old_user_weight + new_user_weight;
    }

    weights_repository_mut(deps.branch()).set_total_weight(total_weight)?;

    Ok(())
}

/// Called for users that we did not encounter previously.
///
/// Will initialize all their rewards for assets with existing distributions to 0, and set
/// their rewards indices to current global index for each asset.
fn initialize_user_indices(deps: DepsMut, user: Addr) -> DistributorResult<()> {
    let all_global_indices =
        asset_distribution_repository(deps.as_ref()).get_all_global_indices()?;

    user_distribution_repository_mut(deps)
        .initialize_distribution_info(all_global_indices, user)?;

    Ok(())
}

/// Updates user's reward indices for all assets.
///
/// Will calculate newly pending rewards since the last update to the user's reward index until now,
/// using their last weight to calculate the newly accrued rewards.
fn update_user_indices(
    deps: DepsMut,
    user: Addr,
    old_user_weight: Uint128,
) -> DistributorResult<()> {
    let all_global_indices =
        asset_distribution_repository(deps.as_ref()).get_all_global_indices()?;

    user_distribution_repository_mut(deps).update_user_indices(
        user,
        all_global_indices,
        old_user_weight,
    )?;

    Ok(())
}

/// Calculate user's effective rewards weight, given their actual weight and minimum weight for
/// rewards eligibility
pub fn calculate_effective_weight(weight: Uint128, minimum_eligible_weight: Uint128) -> Uint128 {
    if weight >= minimum_eligible_weight {
        weight
    } else {
        Uint128::zero()
    }
}
