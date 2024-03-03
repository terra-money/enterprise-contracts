use crate::cw20_distributions::{Cw20Distribution, CW20_DISTRIBUTIONS};
use crate::native_distributions::{NativeDistribution, NATIVE_DISTRIBUTIONS};
use crate::rewards::calculate_user_reward;
use crate::state::{CW20_GLOBAL_INDICES, ENTERPRISE_CONTRACT, NATIVE_GLOBAL_INDICES};
use crate::user_weights::EFFECTIVE_USER_WEIGHTS;
use common::cw::Context;
use cosmwasm_std::{Addr, Decimal, Deps, Response, StdResult, SubMsg, Uint128};
use cw_asset::{Asset, AssetInfo};
use enterprise_protocol::api::{IsRestrictedUserParams, IsRestrictedUserResponse};
use enterprise_protocol::msg::QueryMsg::IsRestrictedUser;
use funds_distributor_api::api::ClaimRewardsMsg;
use funds_distributor_api::error::DistributorError::Unauthorized;
use funds_distributor_api::error::{DistributorError, DistributorResult};
use funds_distributor_api::response::execute_claim_rewards_response;
use DistributorError::RestrictedUser;

struct UserDistribution {
    pub user_index: Decimal,
    pub pending_rewards: Uint128,
}

impl From<UserDistribution> for (Decimal, Uint128) {
    fn from(item: UserDistribution) -> Self {
        (item.user_index, item.pending_rewards)
    }
}

trait Claimable<A: Clone> {
    fn user_distribution(
        &self,
        user: Addr,
        asset: A,
    ) -> DistributorResult<Option<UserDistribution>>;

    fn reset_user_distribution(
        &mut self,
        user: Addr,
        asset: A,
        global_index: Decimal,
    ) -> DistributorResult<()>;

    fn global_index(&self, asset: A) -> DistributorResult<Decimal>;

    fn asset_info(asset: A) -> AssetInfo;

    fn user_weight(&self, user: Addr) -> DistributorResult<Uint128>;

    fn calculate_and_remove_claimable_rewards(
        &mut self,
        user: Addr,
        assets: Vec<A>,
    ) -> DistributorResult<Vec<Asset>> {
        let user_weight = self.user_weight(user.clone())?;

        let mut rewards: Vec<Asset> = vec![];

        for asset in assets {
            let distribution = self.user_distribution(user.clone(), asset.clone())?;
            let global_index = self.global_index(asset.clone())?;

            // if no rewards for the given asset, just skip
            if global_index.is_zero() {
                continue;
            }

            let reward = calculate_user_reward(global_index, distribution, user_weight)?;

            // if no user rewards due for the given asset, just skip - no need to send or store anything
            if reward.is_zero() {
                continue;
            }

            let reward = Asset::new(Self::asset_info(asset.clone()), reward);
            rewards.push(reward);

            self.reset_user_distribution(user.clone(), asset, global_index)?;
        }

        Ok(rewards)
    }
}

struct NativeClaimable<'a> {
    ctx: Context<'a>,
}

impl<'a> Claimable<String> for NativeClaimable<'a> {
    fn user_distribution(
        &self,
        user: Addr,
        asset: String,
    ) -> DistributorResult<Option<UserDistribution>> {
        let distribution = NATIVE_DISTRIBUTIONS().may_load(self.ctx.deps.storage, (user, asset))?;

        Ok(distribution.map(|it| UserDistribution {
            user_index: it.user_index,
            pending_rewards: it.pending_rewards,
        }))
    }

    fn reset_user_distribution(
        &mut self,
        user: Addr,
        asset: String,
        global_index: Decimal,
    ) -> DistributorResult<()> {
        NATIVE_DISTRIBUTIONS().save(
            self.ctx.deps.storage,
            (user.clone(), asset.clone()),
            &NativeDistribution {
                user: user.clone(),
                denom: asset,
                user_index: global_index,
                pending_rewards: Uint128::zero(),
            },
        )?;
        Ok(())
    }

    fn global_index(&self, asset: String) -> DistributorResult<Decimal> {
        let global_index = NATIVE_GLOBAL_INDICES
            .may_load(self.ctx.deps.storage, asset.clone())?
            .unwrap_or_default();
        Ok(global_index)
    }

    fn asset_info(asset: String) -> AssetInfo {
        AssetInfo::native(asset)
    }

    fn user_weight(&self, user: Addr) -> DistributorResult<Uint128> {
        let weight = EFFECTIVE_USER_WEIGHTS.load(self.ctx.deps.storage, user)?;
        Ok(weight)
    }
}

struct Cw20Claimable<'a> {
    ctx: Context<'a>,
}

impl<'a> Claimable<Addr> for Cw20Claimable<'a> {
    fn user_distribution(
        &self,
        user: Addr,
        asset: Addr,
    ) -> DistributorResult<Option<UserDistribution>> {
        let distribution = CW20_DISTRIBUTIONS().may_load(self.ctx.deps.storage, (user, asset))?;

        Ok(distribution.map(|it| UserDistribution {
            user_index: it.user_index,
            pending_rewards: it.pending_rewards,
        }))
    }

    fn reset_user_distribution(
        &mut self,
        user: Addr,
        asset: Addr,
        global_index: Decimal,
    ) -> DistributorResult<()> {
        CW20_DISTRIBUTIONS().save(
            self.ctx.deps.storage,
            (user.clone(), asset.clone()),
            &Cw20Distribution {
                user: user.clone(),
                cw20_asset: asset,
                user_index: global_index,
                pending_rewards: Uint128::zero(),
            },
        )?;
        Ok(())
    }

    fn global_index(&self, asset: Addr) -> DistributorResult<Decimal> {
        let global_index = CW20_GLOBAL_INDICES
            .may_load(self.ctx.deps.storage, asset.clone())?
            .unwrap_or_default();
        Ok(global_index)
    }

    fn asset_info(asset: Addr) -> AssetInfo {
        AssetInfo::cw20(asset)
    }

    fn user_weight(&self, user: Addr) -> DistributorResult<Uint128> {
        let weight = EFFECTIVE_USER_WEIGHTS.load(self.ctx.deps.storage, user)?;
        Ok(weight)
    }
}

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

    let native_rewards = NativeClaimable { ctx: ctx.branch() }
        .calculate_and_remove_claimable_rewards(user.clone(), msg.native_denoms)?;

    let cw20_assets = msg
        .cw20_assets
        .iter()
        .map(|it| ctx.deps.api.addr_validate(it))
        .collect::<StdResult<Vec<Addr>>>()?;
    let cw20_rewards = Cw20Claimable { ctx: ctx.branch() }
        .calculate_and_remove_claimable_rewards(user.clone(), cw20_assets)?;

    let rewards_arrays = vec![native_rewards, cw20_rewards];

    let mut submsgs = vec![];

    for rewards_array in rewards_arrays {
        for reward in rewards_array {
            submsgs.push(SubMsg::new(reward.transfer_msg(user.clone())?));
        }
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
