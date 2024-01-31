use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::App;
use funds_distributor_api::api::MinimumEligibleWeightResponse;
use funds_distributor_api::error::DistributorResult;
use funds_distributor_api::msg::QueryMsg::MinimumEligibleWeight;

pub trait FundsDistributorContract {
    fn minimum_eligible_weight(&self) -> DistributorResult<Uint128>;
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
}

impl TestFundsDistributorContract<'_> {
    pub fn assert_minimum_eligible_weight(&self, weight: impl Into<Uint128>) {
        assert_eq!(self.minimum_eligible_weight().unwrap(), weight.into());
    }
}
