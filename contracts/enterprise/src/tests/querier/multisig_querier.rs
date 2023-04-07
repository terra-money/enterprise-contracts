use crate::cw3::{Cw3VoterDetail, Cw3VoterListResponse};
use crate::tests::querier::custom_querier::CustomQuerier;
use cosmwasm_std::{
    from_binary, to_binary, Binary, ContractResult, QuerierResult, SystemError, SystemResult,
};
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct MultisigQuerier {
    pub members: HashMap<String, HashMap<String, u64>>,
}

pub(crate) fn _members_to_map(
    members: &[(&str, &[(&str, u64)])],
) -> HashMap<String, HashMap<String, u64>> {
    let mut members_map: HashMap<String, HashMap<String, u64>> = HashMap::new();
    for (contract_addr, members) in members.into_iter() {
        let mut contract_members_map: HashMap<String, u64> = HashMap::new();
        for (addr, weight) in members.into_iter() {
            contract_members_map.insert(addr.to_string(), *weight);
        }

        members_map.insert(contract_addr.to_string(), contract_members_map);
    }
    members_map
}

impl CustomQuerier for MultisigQuerier {
    fn query(&self, contract_addr: &str, msg: &Binary) -> Option<QuerierResult> {
        match from_binary(msg) {
            Ok(cw3::Cw3QueryMsg::ListVoters { .. }) => {
                let members: &HashMap<String, u64> = match self.members.get(contract_addr) {
                    Some(members) => members,
                    None => {
                        return Some(SystemResult::Err(SystemError::InvalidRequest {
                            error: format!(
                                "No member info exists for the contract {}",
                                contract_addr
                            ),
                            request: msg.as_slice().into(),
                        }))
                    }
                };

                let voters = members
                    .into_iter()
                    .map(|(addr, weight)| cw3::VoterDetail {
                        addr: addr.to_string(),
                        weight: *weight,
                    })
                    .collect_vec();

                Some(SystemResult::Ok(ContractResult::Ok(
                    to_binary(&cw3::VoterListResponse { voters }).unwrap(),
                )))
            }
            _ => None,
        }
    }
}
