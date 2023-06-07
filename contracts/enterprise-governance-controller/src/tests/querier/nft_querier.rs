use crate::tests::querier::custom_querier::CustomQuerier;
use cosmwasm_std::{
    from_binary, to_binary, Binary, ContractResult, QuerierResult, SystemError, SystemResult,
};
use cw721::{Cw721QueryMsg, NumTokensResponse, TokensResponse};
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct NftQuerier {
    pub num_tokens: HashMap<String, u64>,
    pub nft_holders: HashMap<String, HashMap<String, Vec<String>>>,
}

pub(crate) fn num_tokens_to_map(num_tokens: &[(&str, u64)]) -> HashMap<String, u64> {
    let mut num_tokens_map: HashMap<String, u64> = HashMap::new();
    for (contract_addr, num_tokens) in num_tokens.into_iter() {
        num_tokens_map.insert(contract_addr.to_string(), *num_tokens);
    }
    num_tokens_map
}

pub(crate) fn nft_holders_to_map(
    nft_holders: &[(&str, &[(&str, &[&str])])],
) -> HashMap<String, HashMap<String, Vec<String>>> {
    let mut nft_holders_map: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
    for (contract_addr, holders) in nft_holders.into_iter() {
        let mut holder_map: HashMap<String, Vec<String>> = HashMap::new();
        for (holder, nfts) in holders.into_iter() {
            let nfts_vec = nfts
                .into_iter()
                .map(|token_id| token_id.to_string())
                .collect_vec();
            holder_map.insert(holder.to_string(), nfts_vec);
        }
        nft_holders_map.insert(contract_addr.to_string(), holder_map);
    }
    nft_holders_map
}

impl CustomQuerier for NftQuerier {
    fn query(&self, contract_addr: &str, msg: &Binary) -> Option<QuerierResult> {
        match from_binary(msg) {
            Ok(Cw721QueryMsg::NumTokens {}) => {
                let num_tokens: &u64 = match self.num_tokens.get(contract_addr) {
                    Some(num_tokens) => num_tokens,
                    None => {
                        return Some(SystemResult::Err(SystemError::InvalidRequest {
                            error: format!(
                                "No num tokens info exists for the contract {}",
                                contract_addr
                            ),
                            request: msg.as_slice().into(),
                        }))
                    }
                };

                Some(SystemResult::Ok(ContractResult::Ok(
                    to_binary(&NumTokensResponse { count: *num_tokens }).unwrap(),
                )))
            }
            Ok(Cw721QueryMsg::Tokens { owner, .. }) => {
                let tokens: Vec<String> = match self.nft_holders.get(contract_addr) {
                    Some(holders) => holders
                        .get(&owner)
                        .map(|vec| vec.clone())
                        .unwrap_or_default(),
                    None => {
                        return Some(SystemResult::Err(SystemError::InvalidRequest {
                            error: format!(
                                "No NFT holder info exists for the contract {}",
                                contract_addr
                            ),
                            request: msg.as_slice().into(),
                        }))
                    }
                };

                Some(SystemResult::Ok(ContractResult::Ok(
                    to_binary(&TokensResponse { tokens }).unwrap(),
                )))
            }
            _ => None,
        }
    }
}
