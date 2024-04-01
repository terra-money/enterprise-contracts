use crate::asset_types::{to_reward_assets, RewardAsset};
use crate::repository::era_repository::{get_current_era, set_user_last_claimed_era, FIRST_ERA};
use crate::repository::user_distribution_repository::{
    user_distribution_repository_mut, UserDistributionInfo, UserDistributionRepositoryMut,
};
use crate::rewards;
use common::cw::Context;
use cosmwasm_std::{Addr, DepsMut, Response, StdResult, SubMsg, Uint128};
use cw_asset::{Asset, AssetInfo};
use funds_distributor_api::api::ClaimRewardsMsg;
use funds_distributor_api::api::DistributionType::{Membership, Participation};
use funds_distributor_api::error::DistributorError::Unauthorized;
use funds_distributor_api::error::DistributorResult;
use funds_distributor_api::response::execute_claim_rewards_response;
use rewards::calculate_claimable_rewards;
use std::collections::HashMap;
use std::thread::current;
use RewardAsset::{Cw20, Native};

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

    let mut rewards: HashMap<RewardAsset, Uint128> = HashMap::new();

    for (asset, era, reward, global_index) in claimable_rewards {
        // TODO: duplicates will be present
        // let reward = Asset::new(AssetInfo::from(&asset), reward);

        let new_amount = match rewards.get(&asset) {
            Some(value) => value.checked_add(reward)?,
            None => reward,
        };
        rewards.insert(asset.clone(), new_amount);

        // TODO: what happens when we add another enum value?
        for distribution_type in [Membership, Participation] {
            user_distribution_repository_mut(deps.branch(), distribution_type)
                .set_distribution_info(
                    asset.clone(),
                    user.clone(),
                    era,
                    UserDistributionInfo {
                        user_index: global_index,
                        pending_rewards: Uint128::zero(),
                    },
                )?;
        }
    }

    let current_era = get_current_era(deps.as_ref())?;
    if current_era > FIRST_ERA {
        set_user_last_claimed_era(deps, user, current_era - 1)?;
    }

    Ok(rewards
        .into_iter()
        .map(|(asset, amount)| Asset::new(AssetInfo::from(asset), amount))
        .collect())
}
