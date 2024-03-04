use crate::asset_types::RewardAsset::{Cw20, Native};
use crate::asset_types::{to_reward_assets, RewardAsset};
use crate::repository::asset_repository::{
    asset_distribution_repository, AssetDistributionRepository,
};
use crate::repository::user_distribution_repository::{
    user_distribution_repository, UserDistributionRepository,
};
use crate::repository::weights_repository::weights_repository;
use common::cw::QueryContext;
use cosmwasm_std::{Addr, Decimal, Deps, Fraction, Uint128};
use funds_distributor_api::api::DistributionType::Membership;
use funds_distributor_api::api::{
    Cw20Reward, NativeReward, UserRewardsParams, UserRewardsResponse,
};
use funds_distributor_api::error::DistributorResult;

/// Calculates user's currently available rewards for an asset, given its current global index
/// and user's weight.
pub fn calculate_user_reward(
    global_index: Decimal,
    distribution: Option<impl Into<(Decimal, Uint128)>>,
    user_weight: Uint128,
) -> DistributorResult<Uint128> {
    let (user_index, pending_rewards) =
        distribution.map_or((Decimal::zero(), Uint128::zero()), |it| it.into());

    let user_reward = calculate_new_user_reward(global_index, user_index, user_weight)?
        .checked_add(pending_rewards)?;

    Ok(user_reward)
}

/// Calculates reward accrued for the given asset since the last update to the user's reward
/// index for the given asset.
fn calculate_new_user_reward(
    global_index: Decimal,
    user_index: Decimal,
    user_weight: Uint128,
) -> DistributorResult<Uint128> {
    let user_index_diff = global_index.checked_sub(user_index)?;
    let new_user_reward = user_weight
        .checked_multiply_ratio(user_index_diff.numerator(), user_index_diff.denominator())?;

    Ok(new_user_reward)
}

pub fn query_user_rewards(
    qctx: QueryContext,
    params: UserRewardsParams,
) -> DistributorResult<UserRewardsResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    let assets = to_reward_assets(qctx.deps, params.native_denoms, params.cw20_assets)?;

    query_rewards(qctx.deps, user, assets)
}

fn query_rewards(
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

pub fn calculate_claimable_rewards(
    deps: Deps,
    user: Addr,
    assets: Vec<RewardAsset>,
) -> DistributorResult<Vec<(RewardAsset, Uint128, Decimal)>> {
    // TODO: calculate rewards for both types
    let user_weight = weights_repository(deps, Membership)
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
