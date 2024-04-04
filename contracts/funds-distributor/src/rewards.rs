use crate::asset_types::RewardAsset::{Cw20, Native};
use crate::asset_types::{to_reward_assets, RewardAsset};
use crate::repository::era_repository::{
    get_current_era, get_user_first_era_with_weight, get_user_last_fully_claimed_era,
};
use crate::repository::global_indices_repository::{
    global_indices_repository, GlobalIndicesRepository,
};
use crate::repository::user_distribution_repository::{
    user_distribution_repository, UserDistributionRepository,
};
use crate::repository::weights_repository::weights_repository;
use crate::state::EraId;
use common::cw::QueryContext;
use cosmwasm_std::{Addr, Decimal, Deps, Fraction, Uint128};
use funds_distributor_api::api::DistributionType::{Membership, Participation};
use funds_distributor_api::api::{
    Cw20Reward, DistributionType, NativeReward, UserRewardsParams, UserRewardsResponse,
};
use funds_distributor_api::error::DistributorResult;
use std::collections::HashMap;

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
    let mut native_rewards: HashMap<String, Uint128> = HashMap::new();
    let mut cw20_rewards: HashMap<Addr, Uint128> = HashMap::new();

    for distribution_type in [Membership, Participation] {
        let claimable_rewards =
            calculate_claimable_rewards(deps, user.clone(), assets.clone(), distribution_type)?;

        for (asset, _, amount, _) in claimable_rewards {
            match asset {
                Native { denom } => match native_rewards.get(&denom) {
                    None => {
                        native_rewards.insert(denom, amount);
                    }
                    Some(value) => {
                        native_rewards.insert(denom, amount.checked_add(*value)?);
                    }
                },
                Cw20 { addr } => match cw20_rewards.get(&addr) {
                    None => {
                        cw20_rewards.insert(addr, amount);
                    }
                    Some(value) => {
                        cw20_rewards.insert(addr, amount.checked_add(*value)?);
                    }
                },
            }
        }
    }

    Ok(UserRewardsResponse {
        native_rewards: native_rewards
            .into_iter()
            .map(|(denom, amount)| NativeReward { denom, amount })
            .collect(),
        cw20_rewards: cw20_rewards
            .into_iter()
            .map(|(asset, amount)| Cw20Reward {
                asset: asset.to_string(),
                amount,
            })
            .collect(),
    })
}

pub fn calculate_claimable_rewards(
    deps: Deps,
    user: Addr,
    assets: Vec<RewardAsset>,
    distribution_type: DistributionType,
) -> DistributorResult<Vec<(RewardAsset, EraId, Uint128, Decimal)>> {
    let mut rewards: Vec<(RewardAsset, EraId, Uint128, Decimal)> = vec![];

    let current_era = get_current_era(deps, distribution_type.clone())?;

    let last_claimed_era =
        get_user_last_fully_claimed_era(deps, user.clone(), distribution_type.clone())?;
    let first_relevant_era = match last_claimed_era {
        Some(last_claimed_era) => last_claimed_era,
        None => {
            let first_era_with_weight =
                get_user_first_era_with_weight(deps, user.clone(), distribution_type.clone())?;
            match first_era_with_weight {
                Some(first_era_with_weight) => first_era_with_weight,
                None => {
                    let mut rewards = vec![];
                    for asset in assets {
                        let global_index =
                            global_indices_repository(deps, distribution_type.clone())
                                .get_global_index(asset.clone(), current_era)?
                                .unwrap_or_default();
                        rewards.push((asset, current_era, Uint128::zero(), global_index))
                    }
                    return Ok(rewards);
                }
            }
        }
    };

    for era in first_relevant_era..=current_era {
        let user_weight = weights_repository(deps, distribution_type.clone())
            .get_user_weight(user.clone(), era)?
            .unwrap_or_default();

        for asset in &assets {
            let distribution = user_distribution_repository(deps, distribution_type.clone())
                .get_distribution_info(asset.clone(), user.clone(), era)?;
            let global_index = global_indices_repository(deps, distribution_type.clone())
                .get_global_index(asset.clone(), era)?
                .unwrap_or_default();

            // if no rewards for the given asset, just skip
            if global_index.is_zero() {
                continue;
            }

            // TODO: no need to calculate for every era - old eras have nothing to calculate, just check pending rewards
            let reward = calculate_user_reward(global_index, distribution, user_weight)?;

            // if no user rewards due for the given asset, just skip - no need to send or store anything
            if reward.is_zero() {
                continue;
            }

            // TODO: don't just push them, this will result in duplicates because we're iterating through eras
            rewards.push((asset.clone(), era, reward, global_index));
        }
    }

    Ok(rewards)
}
