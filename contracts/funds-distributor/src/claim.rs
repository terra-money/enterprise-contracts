use crate::cw20_distributions::{Cw20Distribution, CW20_DISTRIBUTIONS};
use crate::native_distributions::{NativeDistribution, NATIVE_DISTRIBUTIONS};
use crate::rewards::calculate_user_reward;
use crate::state::{CW20_GLOBAL_INDICES, NATIVE_GLOBAL_INDICES};
use crate::user_weights::USER_WEIGHTS;
use common::cw::Context;
use cosmwasm_std::{Response, SubMsg, Uint128};
use cw_asset::Asset;
use funds_distributor_api::api::ClaimRewardsMsg;
use funds_distributor_api::error::DistributorResult;

pub fn claim_rewards(ctx: &mut Context, msg: ClaimRewardsMsg) -> DistributorResult<Response> {
    let user = ctx.deps.api.addr_validate(&msg.user)?;

    let user_weight = USER_WEIGHTS
        .may_load(ctx.deps.storage, user.clone())?
        .unwrap_or_default();

    let mut submsgs: Vec<SubMsg> = vec![];

    for denom in msg.native_denoms {
        let distribution =
            NATIVE_DISTRIBUTIONS().may_load(ctx.deps.storage, (user.clone(), denom.clone()))?;
        let global_index = NATIVE_GLOBAL_INDICES
            .may_load(ctx.deps.storage, denom.clone())?
            .unwrap_or_default();

        if global_index.is_zero() {
            continue;
        }

        let reward = calculate_user_reward(global_index, distribution, user_weight);

        if !reward.is_zero() {
            let submsg = Asset::native(denom.clone(), reward).transfer_msg(user.clone())?;
            submsgs.push(SubMsg::new(submsg));
        }

        NATIVE_DISTRIBUTIONS().save(
            ctx.deps.storage,
            (user.clone(), denom.clone()),
            &NativeDistribution {
                user: user.clone(),
                denom,
                user_index: global_index,
                pending_rewards: Uint128::zero(),
            },
        )?;
    }

    for asset in msg.cw20_assets {
        let asset = ctx.deps.api.addr_validate(&asset)?;

        let distribution =
            CW20_DISTRIBUTIONS().may_load(ctx.deps.storage, (user.clone(), asset.clone()))?;
        let global_index = CW20_GLOBAL_INDICES
            .may_load(ctx.deps.storage, asset.clone())?
            .unwrap_or_default();

        if global_index.is_zero() {
            continue;
        }

        let reward = calculate_user_reward(global_index, distribution, user_weight);

        if !reward.is_zero() {
            let submsg = Asset::cw20(asset.clone(), reward).transfer_msg(user.clone())?;
            submsgs.push(SubMsg::new(submsg));
        }

        CW20_DISTRIBUTIONS().save(
            ctx.deps.storage,
            (user.clone(), asset.clone()),
            &Cw20Distribution {
                user: user.clone(),
                cw20_asset: asset,
                user_index: global_index,
                pending_rewards: Uint128::zero(),
            },
        )?;
    }

    Ok(Response::new()
        .add_attribute("action", "claim_rewards")
        .add_attribute("user", user.to_string())
        .add_submessages(submsgs))
}
