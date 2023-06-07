use crate::tests::querier::custom_querier::CustomQuerier;
use cosmwasm_std::{
    from_binary, to_binary, Binary, ContractResult, QuerierResult, SystemError, SystemResult,
    Uint128,
};
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg, TokenInfoResponse};
use cw20_base::state::TokenInfo;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct TokenQuerier {
    pub balances: HashMap<String, HashMap<String, Uint128>>,
    pub token_infos: HashMap<String, TokenInfo>,
}

pub(crate) fn balances_to_map(
    balances: &[(&str, &[(&str, Uint128)])],
) -> HashMap<String, HashMap<String, Uint128>> {
    let mut balances_map: HashMap<String, HashMap<String, Uint128>> = HashMap::new();
    for (contract_addr, balances) in balances.into_iter() {
        let mut contract_balances_map: HashMap<String, Uint128> = HashMap::new();
        for (addr, balance) in balances.into_iter() {
            contract_balances_map.insert(addr.to_string(), *balance);
        }

        balances_map.insert(contract_addr.to_string(), contract_balances_map);
    }
    balances_map
}

pub(crate) fn token_infos_to_map(token_infos: &[(&str, &TokenInfo)]) -> HashMap<String, TokenInfo> {
    let mut token_infos_map: HashMap<String, TokenInfo> = HashMap::new();
    for (contract_addr, token_info) in token_infos.into_iter() {
        let token_info = TokenInfo {
            name: token_info.name.clone(),
            symbol: token_info.symbol.clone(),
            decimals: token_info.decimals,
            total_supply: token_info.total_supply,
            mint: token_info.mint.clone(),
        };
        token_infos_map.insert(contract_addr.to_string(), token_info);
    }
    token_infos_map
}

impl CustomQuerier for TokenQuerier {
    fn query(&self, contract_addr: &str, msg: &Binary) -> Option<QuerierResult> {
        match from_binary(msg) {
            Ok(Cw20QueryMsg::Balance { address }) => {
                let balances: &HashMap<String, Uint128> = match self.balances.get(contract_addr) {
                    Some(balances) => balances,
                    None => {
                        return Some(SystemResult::Err(SystemError::InvalidRequest {
                            error: format!(
                                "No balance info exists for the contract {}",
                                contract_addr
                            ),
                            request: msg.as_slice().into(),
                        }))
                    }
                };

                let balance = match balances.get(&address) {
                    Some(v) => *v,
                    None => {
                        return Some(SystemResult::Ok(ContractResult::Ok(
                            to_binary(&Cw20BalanceResponse {
                                balance: Uint128::zero(),
                            })
                            .unwrap(),
                        )));
                    }
                };

                Some(SystemResult::Ok(ContractResult::Ok(
                    to_binary(&Cw20BalanceResponse { balance }).unwrap(),
                )))
            }
            Ok(Cw20QueryMsg::TokenInfo {}) => {
                let token_info: &TokenInfo = match self.token_infos.get(contract_addr) {
                    Some(token_info) => token_info,
                    None => {
                        return Some(SystemResult::Err(SystemError::InvalidRequest {
                            error: format!(
                                "No token info exists for the contract {}",
                                contract_addr
                            ),
                            request: msg.as_slice().into(),
                        }))
                    }
                };

                Some(SystemResult::Ok(ContractResult::Ok(
                    to_binary(&TokenInfoResponse {
                        name: token_info.name.clone(),
                        symbol: token_info.name.clone(),
                        decimals: token_info.decimals,
                        total_supply: token_info.total_supply,
                    })
                    .unwrap(),
                )))
            }
            _ => None,
        }
    }
}
