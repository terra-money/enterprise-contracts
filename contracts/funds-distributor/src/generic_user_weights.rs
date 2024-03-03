// TODO: rename this file por favor

use crate::repository::asset_repository::{
    asset_distribution_repository, AssetDistributionRepository,
};
use crate::repository::user_distribution_repository::{
    user_distribution_repository_mut, UserDistributionRepositoryMut,
};
use crate::repository::weights_repository::{
    weights_repository, weights_repository_mut, WeightsRepository, WeightsRepositoryMut,
};
use cosmwasm_std::{Addr, DepsMut, Uint128};
use funds_distributor_api::api::UpdateUserWeightsMsg;
use funds_distributor_api::error::DistributorResult;

pub fn update_user_weights(mut deps: DepsMut, msg: UpdateUserWeightsMsg) -> DistributorResult<()> {
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

fn initialize_user_indices(deps: DepsMut, user: Addr) -> DistributorResult<()> {
    let all_global_indices =
        asset_distribution_repository(deps.as_ref()).get_all_global_indices()?;

    user_distribution_repository_mut(deps)
        .initialize_distribution_info(all_global_indices, user)?;

    Ok(())
}

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
