use cosmwasm_std::Uint128;
use cw20::{Cw20Contract, MinterResponse};
use cw_multi_test::App;

pub struct Cw20Assert<'a> {
    pub app: &'a App,
    pub cw20_contract: &'a Cw20Contract,
}

impl Cw20Assert<'_> {
    pub fn balance(&self, user: impl Into<String>, balance: impl Into<Uint128>) {
        assert_eq!(
            self.cw20_contract.balance(&self.app.wrap(), user).unwrap(),
            balance.into(),
        );
    }

    pub fn minter(&self, minter: impl Into<String>, cap: Option<u32>) {
        assert_eq!(
            self.cw20_contract
                .minter(&self.app.wrap())
                .unwrap()
                .unwrap(),
            MinterResponse {
                minter: minter.into(),
                cap: cap.map(Uint128::from)
            },
        );
    }
}
