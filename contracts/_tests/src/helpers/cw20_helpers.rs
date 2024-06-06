use cosmwasm_std::Uint128;
use cw20::Cw20QueryMsg::MarketingInfo;
use cw20::{Cw20Contract, MarketingInfoResponse, MinterResponse, TokenInfoResponse};
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

    pub fn balances(&self, balances: Vec<(impl Into<String>, impl Into<Uint128>)>) {
        for (user, balance) in balances {
            self.balance(user, balance)
        }
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

    pub fn token_info(
        &self,
        name: impl Into<String>,
        symbol: impl Into<String>,
        decimals: u8,
        total_supply: impl Into<Uint128>,
    ) {
        assert_eq!(
            self.cw20_contract.meta(&self.app.wrap()).unwrap(),
            TokenInfoResponse {
                name: name.into(),
                symbol: symbol.into(),
                decimals,
                total_supply: total_supply.into(),
            },
        );
    }

    pub fn marketing_info(&self, info: MarketingInfoResponse) {
        let marketing_info: MarketingInfoResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.cw20_contract.addr().to_string(), &MarketingInfo {})
            .unwrap();
        assert_eq!(marketing_info, info)
    }
}
