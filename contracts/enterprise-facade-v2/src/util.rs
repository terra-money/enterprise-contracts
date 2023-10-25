use cosmwasm_std::{Addr, Coin};
use enterprise_facade_api::api::{AdaptedMsg, AdapterResponse};

pub fn adapter_response_single_msg(
    target_contract: Addr,
    msg: String,
    funds: Vec<Coin>,
) -> AdapterResponse {
    AdapterResponse {
        msgs: vec![AdaptedMsg {
            target_contract,
            msg,
            funds,
        }],
    }
}
