// TODO: rename this file

use crate::asset_types::RewardAsset;
use crate::claim::is_restricted_user;
use crate::repository::asset_repository::{
    asset_distribution_repository, AssetDistributionRepository,
};
use crate::repository::user_distribution_repository::{
    user_distribution_repository_mut, UserDistributionInfo, UserDistributionRepository,
    UserDistributionRepositoryMut,
};
use crate::repository::weights_repository::{weights_repository, WeightsRepository};
use crate::rewards::calculate_user_reward;
use common::cw::Context;
use cosmwasm_std::{Addr, DepsMut, Uint128};
use cw_asset::{Asset, AssetInfo};
use funds_distributor_api::error::DistributorError::{RestrictedUser, Unauthorized};
use funds_distributor_api::error::DistributorResult;

/// Calculates claims, updates internal state, and returns the assets to be sent to the user.
pub fn claim(
    ctx: &mut Context,
    user: Addr,
    assets: Vec<RewardAsset>,
) -> DistributorResult<Vec<Asset>> {
    if is_restricted_user(ctx.deps.as_ref(), user.to_string())? {
        return Err(RestrictedUser);
    }

    if ctx.info.sender != user {
        return Err(Unauthorized);
    }

    let rewards = calculate_and_remove_claimable_rewards(ctx.deps.branch(), user, assets)?;

    Ok(rewards)
}

fn calculate_and_remove_claimable_rewards(
    mut deps: DepsMut,
    user: Addr,
    assets: Vec<RewardAsset>,
) -> DistributorResult<Vec<Asset>> {
    let user_weight = weights_repository(deps.as_ref())
        .get_user_weight(user.clone())?
        .unwrap_or_default();

    let mut rewards: Vec<Asset> = vec![];

    for asset in assets {
        let distribution = user_distribution_repository_mut(deps.branch())
            .get_distribution_info(asset.clone(), user.clone())?;
        let global_index = asset_distribution_repository(deps.as_ref())
            .get_global_index(asset.clone())?
            .unwrap_or_default();

        // if no rewards for the given asset, just skip
        if global_index.is_zero() {
            continue;
        }

        let reward = calculate_user_reward(global_index, distribution, user_weight)?;

        // if no user rewards due for the given asset, just skip - no need to send or store anything
        if reward.is_zero() {
            continue;
        }

        let reward = Asset::new(AssetInfo::from(&asset), reward);
        rewards.push(reward);

        user_distribution_repository_mut(deps.branch()).set_distribution_info(
            asset,
            user.clone(),
            UserDistributionInfo {
                user_index: global_index,
                pending_rewards: Uint128::zero(),
            },
        )?;
    }

    Ok(rewards)
}
