use crate::cw20_distributions::{Cw20Distribution, CW20_DISTRIBUTIONS};
use crate::native_distributions::{NativeDistribution, NATIVE_DISTRIBUTIONS};
use crate::rewards::calculate_user_reward;
use crate::state::{CW20_GLOBAL_INDICES, ENTERPRISE_CONTRACT, NATIVE_GLOBAL_INDICES};
use crate::user_weights::EFFECTIVE_USER_WEIGHTS;
use common::cw::Context;
use cosmwasm_std::{Deps, Response, SubMsg, Uint128};
use cw_asset::Asset;
use enterprise_protocol::api::{IsRestrictedUserParams, IsRestrictedUserResponse};
use enterprise_protocol::msg::QueryMsg::IsRestrictedUser;
use funds_distributor_api::api::ClaimRewardsMsg;
use funds_distributor_api::error::DistributorError::Unauthorized;
use funds_distributor_api::error::{DistributorError, DistributorResult};
use funds_distributor_api::response::execute_claim_rewards_response;
use DistributorError::RestrictedUser;

/// Attempt to claim rewards for the given parameters.
///
/// Calculates rewards currently available to the user, and marks them as claimed.
///
/// Returns a Response containing submessages that will send available rewards to the user.
pub fn claim_rewards(ctx: &mut Context, msg: ClaimRewardsMsg) -> DistributorResult<Response> {
    if is_restricted_user(ctx.deps.as_ref(), msg.user.clone())? {
        return Err(RestrictedUser);
    }

    let user = ctx.deps.api.addr_validate(&msg.user)?;

    if ctx.info.sender != user {
        return Err(Unauthorized);
    }

    let user_weight = EFFECTIVE_USER_WEIGHTS
        .may_load(ctx.deps.storage, user.clone())?
        .unwrap_or_default();

    let mut submsgs: Vec<SubMsg> = vec![];

    for denom in msg.native_denoms {
        let distribution =
            NATIVE_DISTRIBUTIONS().may_load(ctx.deps.storage, (user.clone(), denom.clone()))?;
        let global_index = NATIVE_GLOBAL_INDICES
            .may_load(ctx.deps.storage, denom.clone())?
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

        let submsg = Asset::native(denom.clone(), reward).transfer_msg(user.clone())?;
        submsgs.push(SubMsg::new(submsg));

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

        // if no rewards for the given asset, just skip
        if global_index.is_zero() {
            continue;
        }

        let reward = calculate_user_reward(global_index, distribution, user_weight)?;

        // if no user rewards due for the given asset, just skip - no need to send or store anything
        if reward.is_zero() {
            continue;
        }

        let submsg = Asset::cw20(asset.clone(), reward).transfer_msg(user.clone())?;
        submsgs.push(SubMsg::new(submsg));

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

    Ok(execute_claim_rewards_response(user.to_string()).add_submessages(submsgs))
}

fn is_restricted_user(deps: Deps, user: String) -> DistributorResult<bool> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(deps.storage)?;

    let is_restricted_user: IsRestrictedUserResponse = deps.querier.query_wasm_smart(
        enterprise_contract.to_string(),
        &IsRestrictedUser(IsRestrictedUserParams { user }),
    )?;

    Ok(is_restricted_user.is_restricted)
}
