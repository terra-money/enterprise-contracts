use crate::asset_types::{to_reward_assets, RewardAsset};
use crate::repository::user_distribution_repository::{
    user_distribution_repository_mut, UserDistributionInfo, UserDistributionRepositoryMut,
};
use crate::rewards;
use crate::state::ENTERPRISE_CONTRACT;
use common::cw::Context;
use cosmwasm_std::{Addr, Deps, DepsMut, Response, StdResult, SubMsg, Uint128};
use cw_asset::{Asset, AssetInfo};
use enterprise_protocol::api::{IsRestrictedUserParams, IsRestrictedUserResponse};
use enterprise_protocol::msg::QueryMsg::IsRestrictedUser;
use funds_distributor_api::api::ClaimRewardsMsg;
use funds_distributor_api::api::DistributionType::{Membership, Participation};
use funds_distributor_api::error::DistributorError::Unauthorized;
use funds_distributor_api::error::{DistributorError, DistributorResult};
use funds_distributor_api::response::execute_claim_rewards_response;
use rewards::calculate_claimable_rewards;
use DistributorError::RestrictedUser;

/// Attempt to claim rewards for the given parameters.
///
/// Calculates rewards currently available to the user, and marks them as claimed.
///
/// Returns a Response containing submessages that will send available rewards to the user.
pub fn claim_rewards(ctx: &mut Context, msg: ClaimRewardsMsg) -> DistributorResult<Response> {
    let user = ctx.deps.api.addr_validate(&msg.user)?;

    if ctx.info.sender != user {
        return Err(Unauthorized);
    }

    if is_restricted_user(ctx.deps.as_ref(), msg.user.clone())? {
        return Err(RestrictedUser);
    }

    let assets = to_reward_assets(ctx.deps.as_ref(), msg.native_denoms, msg.cw20_assets)?;

    let rewards = calculate_and_remove_claimable_rewards(ctx.deps.branch(), user.clone(), assets)?;

    let submsgs = rewards
        .into_iter()
        .map(|asset| asset.transfer_msg(user.clone()).map(SubMsg::new))
        .collect::<StdResult<Vec<SubMsg>>>()?;

    Ok(execute_claim_rewards_response(user.to_string()).add_submessages(submsgs))
}

fn calculate_and_remove_claimable_rewards(
    mut deps: DepsMut,
    user: Addr,
    assets: Vec<RewardAsset>,
) -> DistributorResult<Vec<Asset>> {
    // TODO: sort this out, there is a more generic way to accomplish this
    let mut claimable_rewards =
        calculate_claimable_rewards(deps.as_ref(), user.clone(), assets.clone(), Membership)?;
    let mut claimable_rewards_participation =
        calculate_claimable_rewards(deps.as_ref(), user.clone(), assets, Participation)?;

    claimable_rewards.append(&mut claimable_rewards_participation);

    let mut rewards = vec![];

    for (asset, reward, global_index) in claimable_rewards {
        // TODO: duplicates will be present
        let reward = Asset::new(AssetInfo::from(&asset), reward);
        rewards.push(reward);

        // TODO: what happens when we add another enum value?
        for distribution_type in [Membership, Participation] {
            user_distribution_repository_mut(deps.branch(), distribution_type)
                .set_distribution_info(
                    asset.clone(),
                    user.clone(),
                    UserDistributionInfo {
                        user_index: global_index,
                        pending_rewards: Uint128::zero(),
                    },
                )?;
        }
    }

    Ok(rewards)
}

fn is_restricted_user(deps: Deps, user: String) -> DistributorResult<bool> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(deps.storage)?;

    let is_restricted_user: IsRestrictedUserResponse = deps.querier.query_wasm_smart(
        enterprise_contract.to_string(),
        &IsRestrictedUser(IsRestrictedUserParams { user }),
    )?;

    Ok(is_restricted_user.is_restricted)
}
