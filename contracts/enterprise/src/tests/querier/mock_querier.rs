use crate::tests::querier::custom_querier::CustomQuerier;
use crate::tests::querier::enterprise_factory_querier::{
    enterprise_code_ids_to_map, EnterpriseFactoryQuerier,
};
use crate::tests::querier::multisig_querier::{MultisigQuerier, _members_to_map};
use crate::tests::querier::nft_querier::{nft_holders_to_map, num_tokens_to_map, NftQuerier};
use crate::tests::querier::token_querier::{balances_to_map, token_infos_to_map, TokenQuerier};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_slice, Empty, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError, SystemResult,
    Uint128, WasmQuery,
};
use cw20_base::state::TokenInfo;
use std::marker::PhantomData;

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, &[])]));

    OwnedDeps {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<Empty>,
    token_querier: TokenQuerier,
    nft_querier: NftQuerier,
    multisig_querier: MultisigQuerier,
    enterprise_factory_querier: EnterpriseFactoryQuerier,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                let queriers: &[&dyn CustomQuerier] = &[
                    &self.token_querier,
                    &self.nft_querier,
                    &self.multisig_querier,
                    &self.enterprise_factory_querier,
                ];
                for querier in queriers {
                    if let Some(result) = querier.query(contract_addr, msg) {
                        return result;
                    }
                }
                panic!("DO NOT ENTER HERE");
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier {
            base,
            token_querier: TokenQuerier::default(),
            nft_querier: NftQuerier::default(),
            multisig_querier: MultisigQuerier::default(),
            enterprise_factory_querier: EnterpriseFactoryQuerier::default(),
        }
    }

    #[allow(dead_code)]
    pub fn with_token_balances(&mut self, balances: &[(&str, &[(&str, Uint128)])]) {
        self.token_querier = TokenQuerier {
            balances: balances_to_map(balances),
            token_infos: self.token_querier.token_infos.clone(),
        };
    }

    pub fn with_token_infos(&mut self, token_infos: &[(&str, &TokenInfo)]) {
        self.token_querier = TokenQuerier {
            balances: self.token_querier.balances.clone(),
            token_infos: token_infos_to_map(token_infos),
        };
    }

    pub fn with_num_tokens(&mut self, num_tokens: &[(&str, u64)]) {
        self.nft_querier = NftQuerier {
            num_tokens: num_tokens_to_map(num_tokens),
            nft_holders: self.nft_querier.nft_holders.clone(),
        };
    }

    pub fn with_nft_holders(&mut self, nft_holders: &[(&str, &[(&str, &[&str])])]) {
        self.nft_querier = NftQuerier {
            num_tokens: self.nft_querier.num_tokens.clone(),
            nft_holders: nft_holders_to_map(nft_holders),
        };
    }

    pub fn _with_multisig_members(&mut self, members: &[(&str, &[(&str, u64)])]) {
        self.multisig_querier = MultisigQuerier {
            members: _members_to_map(members),
        };
    }

    pub fn with_enterprise_code_ids(&mut self, code_ids: &[(&str, &[u64])]) {
        self.enterprise_factory_querier = EnterpriseFactoryQuerier {
            enterprise_code_ids: enterprise_code_ids_to_map(code_ids),
        };
    }
}
