// TODO: rename this file

use crate::asset_types::RewardAsset;
use crate::claim::is_restricted_user;
use crate::repository::asset_repository::{
    asset_distribution_repository, AssetDistributionRepository,
};
use crate::repository::user_distribution_repository::{
    user_distribution_repository, user_distribution_repository_mut, UserDistributionInfo,
    UserDistributionRepository, UserDistributionRepositoryMut,
};
use crate::repository::weights_repository::{weights_repository, WeightsRepository};
use crate::rewards::calculate_user_reward;
use common::cw::Context;
use cosmwasm_std::{Addr, Decimal, Deps, DepsMut, Uint128};
use cw_asset::{Asset, AssetInfo};
use funds_distributor_api::api::{Cw20Reward, NativeReward, UserRewardsResponse};
use funds_distributor_api::error::DistributorError::{RestrictedUser, Unauthorized};
use funds_distributor_api::error::DistributorResult;
use RewardAsset::{Cw20, Native};

/// Calculates claims, updates internal state, and returns the assets to be sent to the user.
pub fn claim(
    ctx: &mut Context,
    user: Addr,
    assets: Vec<RewardAsset>,
) -> DistributorResult<Vec<Asset>> {
    if ctx.info.sender != user {
        return Err(Unauthorized);
    }

    if is_restricted_user(ctx.deps.as_ref(), user.to_string())? {
        return Err(RestrictedUser);
    }

    let rewards = calculate_and_remove_claimable_rewards(ctx.deps.branch(), user, assets)?;

    Ok(rewards)
}

fn calculate_and_remove_claimable_rewards(
    mut deps: DepsMut,
    user: Addr,
    assets: Vec<RewardAsset>,
) -> DistributorResult<Vec<Asset>> {
    let claimable_rewards = calculate_claimable_rewards(deps.as_ref(), user.clone(), assets)?;

    let mut rewards = vec![];

    for (asset, reward, global_index) in claimable_rewards {
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

fn calculate_claimable_rewards(
    deps: Deps,
    user: Addr,
    assets: Vec<RewardAsset>,
) -> DistributorResult<Vec<(RewardAsset, Uint128, Decimal)>> {
    let user_weight = weights_repository(deps)
        .get_user_weight(user.clone())?
        .unwrap_or_default();

    let mut rewards: Vec<(RewardAsset, Uint128, Decimal)> = vec![];

    for asset in assets {
        let distribution = user_distribution_repository(deps)
            .get_distribution_info(asset.clone(), user.clone())?;
        let global_index = asset_distribution_repository(deps)
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

        rewards.push((asset, reward, global_index));
    }

    Ok(rewards)
}

pub fn query_rewards(
    deps: Deps,
    user: Addr,
    assets: Vec<RewardAsset>,
) -> DistributorResult<UserRewardsResponse> {
    let mut native_rewards = vec![];
    let mut cw20_rewards = vec![];

    let claimable_rewards = calculate_claimable_rewards(deps, user, assets)?;

    for (asset, amount, _) in claimable_rewards {
        match asset {
            Native { denom } => native_rewards.push(NativeReward { denom, amount }),
            Cw20 { addr } => cw20_rewards.push(Cw20Reward {
                asset: addr.to_string(),
                amount,
            }),
        }
    }

    Ok(UserRewardsResponse {
        native_rewards,
        cw20_rewards,
    })
}
