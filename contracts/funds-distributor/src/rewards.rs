use crate::cw20_distributions::CW20_DISTRIBUTIONS;
use crate::native_distributions::NATIVE_DISTRIBUTIONS;
use crate::state::{CW20_GLOBAL_INDICES, NATIVE_GLOBAL_INDICES};
use crate::user_weights::EFFECTIVE_USER_WEIGHTS;
use common::cw::QueryContext;
use cosmwasm_std::{Addr, Decimal, Fraction, Uint128};
use funds_distributor_api::api::{
    Cw20Reward, NativeReward, UserRewardsParams, UserRewardsResponse,
};
use funds_distributor_api::error::DistributorResult;
use std::collections::HashSet;

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
pub fn calculate_new_user_reward(
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

    let user_weight = EFFECTIVE_USER_WEIGHTS
        .may_load(qctx.deps.storage, user.clone())?
        .unwrap_or_default();

    let mut native_rewards: Vec<NativeReward> = vec![];

    let mut denom_set: HashSet<String> = HashSet::new();

    for denom in params.native_denoms {
        if denom_set.contains(&denom) {
            continue;
        }

        denom_set.insert(denom.clone());

        let global_index = NATIVE_GLOBAL_INDICES
            .may_load(qctx.deps.storage, denom.clone())?
            .unwrap_or_default();

        let distribution =
            NATIVE_DISTRIBUTIONS().may_load(qctx.deps.storage, (user.clone(), denom.clone()))?;

        let reward = calculate_user_reward(global_index, distribution, user_weight)?;

        native_rewards.push(NativeReward {
            denom,
            amount: reward,
        });
    }

    let mut cw20_rewards: Vec<Cw20Reward> = vec![];

    let mut asset_set: HashSet<Addr> = HashSet::new();

    for asset in params.cw20_assets {
        let asset = qctx.deps.api.addr_validate(&asset)?;

        if asset_set.contains(&asset) {
            continue;
        }

        asset_set.insert(asset.clone());

        let global_index = CW20_GLOBAL_INDICES
            .may_load(qctx.deps.storage, asset.clone())?
            .unwrap_or_default();

        let distribution =
            CW20_DISTRIBUTIONS().may_load(qctx.deps.storage, (user.clone(), asset.clone()))?;

        let reward = calculate_user_reward(global_index, distribution, user_weight)?;

        cw20_rewards.push(Cw20Reward {
            asset: asset.to_string(),
            amount: reward,
        });
    }

    Ok(UserRewardsResponse {
        native_rewards,
        cw20_rewards,
    })
}
