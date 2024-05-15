use crate::participation::pre_user_votes_change;
use crate::repository::era_repository::{
    get_current_era, get_user_first_era_with_weight, get_user_last_resolved_era,
    set_user_first_era_with_weight_if_empty, set_user_last_resolved_era, FIRST_ERA,
};
use crate::repository::global_indices_repository::{
    global_indices_repository, GlobalIndicesRepository,
};
use crate::repository::user_distribution_repository::{
    user_distribution_repository_mut, UserDistributionRepositoryMut,
};
use crate::repository::weights_repository::{weights_repository, weights_repository_mut};
use crate::state::{EraId, ADMIN};
use common::cw::Context;
use cosmwasm_std::{Addr, DepsMut, Response, Uint128};
use cw_storage_plus::Map;
use funds_distributor_api::api::DistributionType::{Membership, Participation};
use funds_distributor_api::api::{
    DistributionType, PreUserVotesChangeMsg, UpdateUserWeightsMsg, UserWeight,
};
use funds_distributor_api::error::DistributorError::Unauthorized;
use funds_distributor_api::error::{DistributorError, DistributorResult};
use funds_distributor_api::response::execute_update_user_weights_response;
use DistributorError::DuplicateInitialWeight;

pub const USER_WEIGHTS: Map<(EraId, Addr), Uint128> = Map::new("user_weights");

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
) -> DistributorResult<()> {
    let mut total_weight =
        weights_repository(ctx.deps.as_ref(), Membership).get_total_weight(FIRST_ERA)?;

    for user_weight in initial_weights {
        let user = ctx.deps.api.addr_validate(&user_weight.user)?;

        set_user_first_era_with_weight_if_empty(
            ctx.deps.branch(),
            user.clone(),
            FIRST_ERA,
            Membership,
        )?;

        let existing_user_weight = weights_repository(ctx.deps.as_ref(), Membership)
            .get_user_weight(user.clone(), FIRST_ERA)?;
        if existing_user_weight.is_some() {
            return Err(DuplicateInitialWeight);
        }

        let effective_user_weight = weights_repository_mut(ctx.deps.branch(), Membership)
            .set_user_weight(user, user_weight.weight)?;

        total_weight += effective_user_weight;
    }

    weights_repository_mut(ctx.deps.branch(), Membership)
        .set_total_weight(total_weight, FIRST_ERA)?;

    weights_repository_mut(ctx.deps.branch(), Participation)
        .set_total_weight(Uint128::zero(), FIRST_ERA)?;

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

    update_user_weights_checked(ctx, msg)?;

    Ok(execute_update_user_weights_response())
}

fn update_user_weights_checked(
    ctx: &mut Context,
    msg: UpdateUserWeightsMsg,
) -> DistributorResult<()> {
    // TODO: check if we need variable distribution type here, we probably do
    let distribution_type = Membership;

    let current_era = get_current_era(ctx.deps.as_ref(), distribution_type.clone())?;

    let mut total_weight = weights_repository(ctx.deps.as_ref(), distribution_type.clone())
        .get_total_weight(current_era)?;

    for user_weight_change in &msg.new_user_weights {
        let user = ctx.deps.api.addr_validate(&user_weight_change.user)?;

        set_user_first_era_with_weight_if_empty(
            ctx.deps.branch(),
            user.clone(),
            current_era,
            Membership,
        )?;

        let old_user_weight = weights_repository(ctx.deps.as_ref(), distribution_type.clone())
            .get_user_weight(user.clone(), current_era)?; // TODO: definitely just the current era?

        match old_user_weight {
            None => {
                // we have not encountered this user, so we need to ensure their distribution
                // indices are set to current global indices
                initialize_user_indices(
                    ctx.deps.branch(),
                    user.clone(),
                    distribution_type.clone(),
                )?;
            }
            Some(old_user_weight) => {
                // the user already had their weight previously, so we use that weight
                // to calculate how many rewards for each asset they've accrued since we last
                // calculated their pending rewards
                update_user_indices(
                    ctx.deps.branch(),
                    user.clone(),
                    old_user_weight,
                    distribution_type.clone(),
                )?;
            }
        };

        let new_user_weight = weights_repository_mut(ctx.deps.branch(), distribution_type.clone())
            .set_user_weight(user.clone(), user_weight_change.weight)?;

        let old_user_weight = old_user_weight.unwrap_or_default();

        total_weight = total_weight - old_user_weight + new_user_weight;
    }

    // TODO: not sure if current era is correct here
    weights_repository_mut(ctx.deps.branch(), distribution_type)
        .set_total_weight(total_weight, current_era)?;

    // TODO: a bit dirty, but will do the trick
    pre_user_votes_change(
        ctx,
        PreUserVotesChangeMsg {
            users: msg.new_user_weights.into_iter().map(|it| it.user).collect(),
        },
    )?;

    Ok(())
}

/// Called for users that we did not encounter previously.
///
/// Will initialize all their rewards for assets with existing distributions to 0, and set
/// their rewards indices to current global index for each asset.
pub fn initialize_user_indices(
    deps: DepsMut,
    user: Addr,
    distribution_type: DistributionType,
) -> DistributorResult<()> {
    let current_era = get_current_era(deps.as_ref(), distribution_type.clone())?;

    let all_global_indices = global_indices_repository(deps.as_ref(), distribution_type.clone())
        .get_all_global_indices(current_era)?;

    // TODO: now that we have eras, this may be incorrect - users may have had non-zero weights from the previous eras
    user_distribution_repository_mut(deps, distribution_type).initialize_distribution_info(
        all_global_indices,
        user,
        current_era,
    )?;

    Ok(())
}

/// Updates user's reward indices for all assets.
///
/// Will calculate newly pending rewards since the last update to the user's reward index until now,
/// using their last weight to calculate the newly accrued rewards.
pub fn update_user_indices(
    mut deps: DepsMut,
    user: Addr,
    // TODO: for participation (at least), this weight isn't constant for all eras that are going to be updated here
    // TODO: for membership, we have to determine what the effective weight was for each era
    old_user_weight: Uint128,
    distribution_type: DistributionType,
) -> DistributorResult<()> {
    let current_era = get_current_era(deps.as_ref(), distribution_type.clone())?;

    let user_last_resolved_era =
        get_user_last_resolved_era(deps.as_ref(), user.clone(), distribution_type.clone())?;
    // TODO: lol rename this value
    let first_era_of_interest = match user_last_resolved_era {
        Some(last_resolved_era) => last_resolved_era + 1,
        None => {
            let first_era_with_weight = get_user_first_era_with_weight(
                deps.as_ref(),
                user.clone(),
                distribution_type.clone(),
            )?;
            match first_era_with_weight {
                Some(era) => era,
                None => {
                    todo!("nothing to update here, right? the user has no weights, we shouldn't even end up here - this case should go to initialize_user_indices?")
                }
            }
        }
    };

    for era in first_era_of_interest..=current_era {
        let all_global_indices =
            global_indices_repository(deps.as_ref(), distribution_type.clone())
                .get_all_global_indices(era)?;

        user_distribution_repository_mut(deps.branch(), distribution_type.clone())
            .update_user_indices(user.clone(), era, all_global_indices, old_user_weight)?;
    }

    if current_era > FIRST_ERA {
        set_user_last_resolved_era(deps.branch(), user, current_era - 1, distribution_type)?;
    }

    Ok(())
}
