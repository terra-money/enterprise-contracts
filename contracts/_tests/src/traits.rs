use std::{collections::BTreeMap, str::FromStr};

use cosmwasm_std::{Addr, Coin, Decimal, Uint128};
use cw_multi_test::{App, BankSudo, SudoMsg};
use enterprise_factory_api::api::CreateDaoMsg;
use enterprise_governance_controller_api::api::CreateProposalMsg;
use poll_engine_api::api::VoteOutcome;

pub trait IntoAddr {
    fn into_addr(self) -> Addr;
}

impl IntoAddr for &str {
    fn into_addr(self) -> Addr {
        Addr::unchecked(self)
    }
}

impl IntoAddr for String {
    fn into_addr(self) -> Addr {
        Addr::unchecked(self)
    }
}

pub trait IntoDecimal {
    fn into_decimal(self) -> Decimal;
}

impl IntoDecimal for &str {
    fn into_decimal(self) -> Decimal {
        Decimal::from_str(self).unwrap()
    }
}

pub trait IntoUint {
    fn into_uint128(self) -> Uint128;
}

impl IntoUint for i32 {
    fn into_uint128(self) -> Uint128 {
        Uint128::from(self as u128)
    }
}

impl IntoUint for u128 {
    fn into_uint128(self) -> Uint128 {
        Uint128::from(self)
    }
}

impl IntoUint for usize {
    fn into_uint128(self) -> Uint128 {
        Uint128::from(self as u128)
    }
}

pub trait IntoExecuteMsg {
    type ExecuteMsg;

    fn into_execute_msg(self) -> Self::ExecuteMsg;
}

impl IntoExecuteMsg for CreateDaoMsg {
    type ExecuteMsg = enterprise_factory_api::msg::ExecuteMsg;
    fn into_execute_msg(self) -> Self::ExecuteMsg {
        enterprise_factory_api::msg::ExecuteMsg::CreateDao(Box::new(self))
    }
}

impl IntoExecuteMsg for CreateProposalMsg {
    type ExecuteMsg = enterprise_governance_controller_api::msg::ExecuteMsg;

    fn into_execute_msg(self) -> Self::ExecuteMsg {
        enterprise_governance_controller_api::msg::ExecuteMsg::CreateProposal(self)
    }
}

pub trait ImplApp {
    fn mint_native(&mut self, balances: Vec<(impl Into<String>, Vec<Coin>)>);
}

impl ImplApp for App {
    fn mint_native(&mut self, balances: Vec<(impl Into<String>, Vec<Coin>)>) {
        for (addr, coins) in balances {
            self.sudo(SudoMsg::Bank(BankSudo::Mint {
                to_address: addr.into(),
                amount: coins,
            }))
            .unwrap();
        }
    }
}

pub trait IntoVoteResult {
    fn yes(&self) -> u128;
    fn no(&self) -> u128;
    fn abstain(&self) -> u128;
    fn veto(&self) -> u128;
}

impl IntoVoteResult for BTreeMap<u8, u128> {
    fn yes(&self) -> u128 {
        self.get(&(VoteOutcome::Yes as u8))
            .copied()
            .unwrap_or_default()
    }

    fn no(&self) -> u128 {
        self.get(&(VoteOutcome::No as u8))
            .copied()
            .unwrap_or_default()
    }

    fn abstain(&self) -> u128 {
        self.get(&(VoteOutcome::Abstain as u8))
            .copied()
            .unwrap_or_default()
    }

    fn veto(&self) -> u128 {
        self.get(&(VoteOutcome::Veto as u8))
            .copied()
            .unwrap_or_default()
    }
}
