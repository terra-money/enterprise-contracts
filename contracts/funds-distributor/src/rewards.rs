use crate::cw20_distributions::{Cw20Distribution, CW20_DISTRIBUTIONS};
use crate::native_distributions::{NativeDistribution, NATIVE_DISTRIBUTIONS};
use crate::state::{CW20_GLOBAL_INDICES, NATIVE_GLOBAL_INDICES};
use common::cw::QueryContext;
use cosmwasm_std::{Decimal, Uint128};
use funds_distributor_api::api::{
    Cw20Reward, NativeReward, UserRewardsParams, UserRewardsResponse,
};
use funds_distributor_api::error::DistributorResult;
use std::ops::{Add, Mul, Sub};

pub fn calculate_native_user_reward(
    global_index: Decimal,
    distribution: Option<NativeDistribution>,
    user_stake: Uint128,
) -> Uint128 {
    let (user_index, pending_rewards) = distribution
        .map_or((Decimal::zero(), Uint128::zero()), |it| {
            (it.user_index, it.pending_rewards)
        });

    calculate_new_user_reward(global_index, user_index, user_stake).add(pending_rewards)
}

pub fn calculate_cw20_user_reward(
    global_index: Decimal,
    distribution: Option<Cw20Distribution>,
    user_stake: Uint128,
) -> Uint128 {
    let (user_index, pending_rewards) = distribution
        .map_or((Decimal::zero(), Uint128::zero()), |it| {
            (it.user_index, it.pending_rewards)
        });

    calculate_new_user_reward(global_index, user_index, user_stake).add(pending_rewards)
}

pub fn calculate_new_user_reward(
    global_index: Decimal,
    user_index: Decimal,
    user_stake: Uint128,
) -> Uint128 {
    // TODO: test whether this rounds down, if not then we could end up insolvent (distributing more rewards than we have in the pool)
    global_index.sub(user_index).mul(user_stake)
}

pub fn query_user_rewards(
    qctx: QueryContext,
    params: UserRewardsParams,
) -> DistributorResult<UserRewardsResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    let mut native_rewards: Vec<NativeReward> = vec![];

    for denom in params.native_denoms {
        let global_index = NATIVE_GLOBAL_INDICES
            .may_load(qctx.deps.storage, denom.clone())?
            .unwrap_or_default();

        let distribution =
            NATIVE_DISTRIBUTIONS().may_load(qctx.deps.storage, (user.clone(), denom.clone()))?;

        let reward = calculate_native_user_reward(global_index, distribution, params.user_stake);

        native_rewards.push(NativeReward {
            denom,
            amount: reward,
        });
    }

    let mut cw20_rewards: Vec<Cw20Reward> = vec![];

    for asset in params.cw20_assets {
        let asset = qctx.deps.api.addr_validate(&asset)?;

        let global_index = CW20_GLOBAL_INDICES
            .may_load(qctx.deps.storage, asset.clone())?
            .unwrap_or_default();

        let distribution =
            CW20_DISTRIBUTIONS().may_load(qctx.deps.storage, (user.clone(), asset.clone()))?;

        let reward = calculate_cw20_user_reward(global_index, distribution, params.user_stake);

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
