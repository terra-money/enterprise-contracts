use crate::repository::asset_repository::{
    asset_distribution_repository, AssetDistributionRepository,
};
use crate::repository::user_distribution_repository::{
    user_distribution_repository_mut, UserDistributionRepositoryMut,
};
use crate::repository::weights_repository::{weights_repository, weights_repository_mut};
use crate::state::ADMIN;
use crate::user_weights::{EFFECTIVE_USER_WEIGHTS, USER_WEIGHTS};
use common::cw::{Context, QueryContext};
use cosmwasm_std::{Addr, DepsMut, Order, Response, StdResult, Uint128};
use cw_storage_plus::Item;
use funds_distributor_api::api::DistributionType::Membership;
use funds_distributor_api::api::{MinimumEligibleWeightResponse, UpdateMinimumEligibleWeightMsg};
use funds_distributor_api::error::DistributorError::Unauthorized;
use funds_distributor_api::error::DistributorResult;
use funds_distributor_api::response::execute_update_minimum_eligible_weight_response;
use itertools::Itertools;
use std::ops::Range;

/// Minimum weight that a user should have to be eligible for receiving rewards.
pub const MINIMUM_ELIGIBLE_WEIGHT: Item<Uint128> = Item::new("minimum_eligible_weight");

pub fn execute_update_minimum_eligible_weight(
    ctx: &mut Context,
    msg: UpdateMinimumEligibleWeightMsg,
) -> DistributorResult<Response> {
    let admin = ADMIN.load(ctx.deps.storage)?;

    if ctx.info.sender != admin {
        return Err(Unauthorized);
    }

    let old_minimum_weight = MINIMUM_ELIGIBLE_WEIGHT.load(ctx.deps.storage)?;
    let new_minimum_weight = msg.minimum_eligible_weight;

    update_minimum_eligible_weight(ctx.deps.branch(), old_minimum_weight, new_minimum_weight)?;

    Ok(execute_update_minimum_eligible_weight_response(
        old_minimum_weight,
        new_minimum_weight,
    ))
}

/// Update minimum eligible weight for rewards by going through all the users
/// between the old and the new minimum and updating their effective weight (to either their
/// actual weight, or 0, depending on whether they're above or below the new minimum).
// TODO: the name is very similar to the above, but this does not check for unauthorized use; reveal this through the name somehow
pub fn update_minimum_eligible_weight(
    mut deps: DepsMut,
    old_minimum_weight: Uint128,
    new_minimum_weight: Uint128,
) -> DistributorResult<()> {
    if old_minimum_weight == new_minimum_weight {
        return Ok(());
    }

    // determine the range of weights that are affected by the change
    let weight_range = if old_minimum_weight < new_minimum_weight {
        // old_min < new_min, we need to change for users with old_min <= weight < new_min
        Range {
            start: old_minimum_weight,
            end: new_minimum_weight,
        }
    } else {
        // old minimum > new minimum, we need to change for users with new_min <= weight < old_min
        Range {
            start: new_minimum_weight,
            end: old_minimum_weight,
        }
    };

    // find all users with weights from the range between old min and new min
    let affected_users_weights = USER_WEIGHTS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(Addr, Uint128)>>>()?
        .into_iter()
        .filter_map(|(user, weight)| {
            if weight_range.contains(&weight) {
                Some((user, weight))
            } else {
                None
            }
        })
        .collect_vec();

    let mut total_weight = weights_repository(deps.as_ref(), Membership).get_total_weight()?;

    // whether effective weights for users should become their actual weights, or zero
    let use_actual_weights = old_minimum_weight > new_minimum_weight;

    // go through all affected users and update their effective weights
    for (user, user_weight) in affected_users_weights {
        let old_effective_weight = EFFECTIVE_USER_WEIGHTS
            .may_load(deps.storage, user.clone())?
            .unwrap_or_default();

        // update the state of user's rewards distributions to current global indices, placing any
        // newly accrued rewards since last updates into their pending rewards
        let all_global_indices =
            asset_distribution_repository(deps.as_ref(), Membership).get_all_global_indices()?;
        user_distribution_repository_mut(deps.branch(), Membership).update_user_indices(
            user.clone(),
            all_global_indices,
            old_effective_weight,
        )?;

        let new_effective_weight = if use_actual_weights {
            user_weight
        } else {
            Uint128::zero()
        };

        // change user's effective weight to account for the change in effective weight
        EFFECTIVE_USER_WEIGHTS.save(deps.storage, user, &new_effective_weight)?;

        // update total weight
        total_weight = total_weight - old_effective_weight + new_effective_weight;
    }

    MINIMUM_ELIGIBLE_WEIGHT.save(deps.storage, &new_minimum_weight)?;

    weights_repository_mut(deps.branch(), Membership).set_total_weight(total_weight)?;

    Ok(())
}

pub fn query_minimum_eligible_weight(
    qctx: QueryContext,
) -> DistributorResult<MinimumEligibleWeightResponse> {
    let minimum_eligible_weight = MINIMUM_ELIGIBLE_WEIGHT.load(qctx.deps.storage)?;

    Ok(MinimumEligibleWeightResponse {
        minimum_eligible_weight,
    })
}
