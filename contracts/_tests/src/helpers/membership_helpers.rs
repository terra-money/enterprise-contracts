use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::App;
use cw_utils::Expiration::Never;
use membership_common_api::api::{
    TotalWeightParams, TotalWeightResponse, UserWeightParams, UserWeightResponse,
};
use membership_common_api::error::MembershipResult;
use membership_common_api::msg::QueryMsg::{TotalWeight, UserWeight};

pub trait MembershipContract {
    fn user_weight(&self, user: impl Into<String>) -> MembershipResult<Uint128>;
    fn total_weight(&self) -> MembershipResult<Uint128>;
}

pub struct TestMembershipContract<'a> {
    pub app: &'a App,
    pub addr: Addr,
}

impl MembershipContract for TestMembershipContract<'_> {
    fn user_weight(&self, user: impl Into<String>) -> MembershipResult<Uint128> {
        let user_weight: UserWeightResponse = self.app.wrap().query_wasm_smart(
            self.addr.to_string(),
            &UserWeight(UserWeightParams { user: user.into() }),
        )?;
        Ok(user_weight.weight)
    }

    fn total_weight(&self) -> MembershipResult<Uint128> {
        let total_weight: TotalWeightResponse = self.app.wrap().query_wasm_smart(
            self.addr.to_string(),
            &TotalWeight(TotalWeightParams {
                expiration: Never {},
            }),
        )?;
        Ok(total_weight.total_weight)
    }
}

impl TestMembershipContract<'_> {
    pub fn assert_user_weight(&self, user: impl Into<String>, weight: u8) {
        let user_weight = self.user_weight(user).unwrap();
        assert_eq!(user_weight, Uint128::from(weight))
    }

    pub fn assert_user_weights(&self, user_weights: Vec<(impl Into<String>, u8)>) {
        for (user, weight) in user_weights {
            self.assert_user_weight(user, weight)
        }
    }

    pub fn assert_total_weight(&self, weight: u8) {
        let total_weight = self.total_weight().unwrap();
        assert_eq!(total_weight, Uint128::from(weight))
    }
}
