use crate::helpers::cw_multitest_helpers::ADMIN;
use crate::traits::ImplApp;
use cosmwasm_std::{coins, wasm_execute, Addr, Uint128};
use cw_multi_test::{App, AppResponse, Executor};
use funds_distributor_api::api::{
    Cw20Reward, MinimumEligibleWeightResponse, NativeReward, UserRewardsParams, UserRewardsResponse,
};
use funds_distributor_api::error::DistributorResult;
use funds_distributor_api::msg::ExecuteMsg::DistributeNative;
use funds_distributor_api::msg::QueryMsg::{MinimumEligibleWeight, UserRewards};
use itertools::Itertools;

pub trait FundsDistributorContract {
    fn minimum_eligible_weight(&self) -> DistributorResult<Uint128>;
    fn user_rewards(
        &self,
        user: impl Into<String>,
        native: Vec<impl Into<String>>,
        cw20: Vec<impl Into<String>>,
    ) -> DistributorResult<UserRewardsResponse>;
}

pub struct TestFundsDistributorContract<'a> {
    pub app: &'a App,
    pub addr: Addr,
}

impl FundsDistributorContract for TestFundsDistributorContract<'_> {
    fn minimum_eligible_weight(&self) -> DistributorResult<Uint128> {
        let minimum_weight_for_rewards: MinimumEligibleWeightResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.addr.to_string(), &MinimumEligibleWeight {})?;
        Ok(minimum_weight_for_rewards.minimum_eligible_weight)
    }

    fn user_rewards(
        &self,
        user: impl Into<String>,
        native: Vec<impl Into<String>>,
        cw20: Vec<impl Into<String>>,
    ) -> DistributorResult<UserRewardsResponse> {
        let native_denoms = native.into_iter().map(|it| it.into()).collect();
        let cw20_assets = cw20.into_iter().map(|it| it.into()).collect();
        let user_rewards: UserRewardsResponse = self.app.wrap().query_wasm_smart(
            self.addr.to_string(),
            &UserRewards(UserRewardsParams {
                user: user.into(),
                native_denoms,
                cw20_assets,
            }),
        )?;
        Ok(user_rewards)
    }
}

impl TestFundsDistributorContract<'_> {
    pub fn assert_minimum_eligible_weight(&self, weight: impl Into<Uint128>) {
        assert_eq!(self.minimum_eligible_weight().unwrap(), weight.into());
    }

    pub fn assert_native_user_rewards(
        &self,
        user: impl Into<String>,
        rewards: Vec<(impl Into<String> + Clone, u128)>,
    ) {
        let expected_rewards = rewards
            .iter()
            .map(|(denom, amount)| NativeReward {
                denom: denom.clone().into(),
                amount: (*amount).into(),
            })
            .collect_vec();

        let native_denoms = rewards.into_iter().map(|(denom, _)| denom).collect();
        let cw20: Vec<String> = vec![];

        assert_eq!(
            self.user_rewards(user, native_denoms, cw20)
                .unwrap()
                .native_rewards,
            expected_rewards
        );
    }

    pub fn assert_cw20_user_rewards(
        &self,
        user: impl Into<String>,
        rewards: Vec<(impl Into<String> + Clone, u128)>,
    ) {
        let expected_rewards = rewards
            .iter()
            .map(|(asset, amount)| Cw20Reward {
                asset: asset.clone().into(),
                amount: (*amount).into(),
            })
            .collect_vec();

        let cw20 = rewards.into_iter().map(|(asset, _)| asset).collect();
        let native: Vec<String> = vec![];

        assert_eq!(
            self.user_rewards(user, native, cw20).unwrap().cw20_rewards,
            expected_rewards
        );
    }
}

impl TestFundsDistributorContract<'_> {
    pub fn distribute_native(
        &self,
        app: &mut App,
        denom: impl Into<String>,
        amount: u128,
    ) -> anyhow::Result<AppResponse> {
        let distributor = ADMIN;

        let coins = coins(amount, denom);

        app.mint_native(vec![(distributor, coins.clone())]);

        let response = app.execute(
            Addr::unchecked(distributor),
            wasm_execute(
                self.addr.to_string(),
                &DistributeNative {
                    distribution_type: None,
                },
                coins,
            )?
            .into(),
        )?;

        Ok(response)
    }
}
