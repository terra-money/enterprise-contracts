use cosmwasm_std::{Binary, QuerierResult};

pub(crate) trait CustomQuerier {
    fn query(&self, contract_addr: &str, msg: &Binary) -> Option<QuerierResult>;
}
