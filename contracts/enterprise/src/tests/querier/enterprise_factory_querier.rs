use crate::tests::querier::custom_querier::CustomQuerier;
use cosmwasm_std::{
    from_binary, to_binary, Binary, ContractResult, QuerierResult, SystemError, SystemResult,
    Uint64,
};
use enterprise_factory_api::api::{EnterpriseCodeIdsResponse, IsEnterpriseCodeIdResponse};
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct EnterpriseFactoryQuerier {
    pub enterprise_code_ids: HashMap<String, Vec<u64>>,
}

pub(crate) fn enterprise_code_ids_to_map(code_ids: &[(&str, &[u64])]) -> HashMap<String, Vec<u64>> {
    let mut code_ids_map: HashMap<String, Vec<u64>> = HashMap::new();
    for (contract_addr, code_ids) in code_ids.into_iter() {
        let mut code_ids_vec: Vec<u64> = vec![];
        for code_id in code_ids.into_iter() {
            code_ids_vec.push(*code_id);
        }

        code_ids_map.insert(contract_addr.to_string(), code_ids_vec);
    }
    code_ids_map
}

impl CustomQuerier for EnterpriseFactoryQuerier {
    fn query(&self, contract_addr: &str, msg: &Binary) -> Option<QuerierResult> {
        match from_binary(msg) {
            Ok(enterprise_factory_api::msg::QueryMsg::IsEnterpriseCodeId(data)) => {
                let is_enterprise_code_id: bool = match self.enterprise_code_ids.get(contract_addr)
                {
                    Some(code_ids) => code_ids.contains(&data.code_id.u64()),
                    None => {
                        return Some(SystemResult::Err(SystemError::InvalidRequest {
                            error: format!(
                                "No enterprise code IDs info exists for the contract {}",
                                contract_addr
                            ),
                            request: msg.as_slice().into(),
                        }))
                    }
                };
                Some(SystemResult::Ok(ContractResult::Ok(
                    to_binary(&IsEnterpriseCodeIdResponse {
                        is_enterprise_code_id,
                    })
                    .unwrap(),
                )))
            }
            Ok(enterprise_factory_api::msg::QueryMsg::EnterpriseCodeIds(..)) => {
                let code_ids: Vec<Uint64> = match self.enterprise_code_ids.get(contract_addr) {
                    Some(code_ids) => code_ids
                        .into_iter()
                        .map(|id| Uint64::from(*id))
                        .collect_vec(),
                    None => {
                        return Some(SystemResult::Err(SystemError::InvalidRequest {
                            error: format!(
                                "No enterprise code IDs info exists for the contract {}",
                                contract_addr
                            ),
                            request: msg.as_slice().into(),
                        }))
                    }
                };
                Some(SystemResult::Ok(ContractResult::Ok(
                    to_binary(&EnterpriseCodeIdsResponse { code_ids }).unwrap(),
                )))
            }
            _ => None,
        }
    }
}
