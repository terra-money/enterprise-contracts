use cosmwasm_std::CosmosMsg::Stargate;
use cosmwasm_std::{to_json_binary, wasm_execute, CosmosMsg, StdResult, Uint128};
use prost::Message;

#[derive(Clone, PartialEq, prost::Message)]
pub struct Coin {
    #[prost(string, tag = "1")]
    pub denom: String,

    #[prost(string, tag = "2")]
    pub amount: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct MsgTransfer {
    #[prost(string, tag = "1")]
    pub source_port: String,

    #[prost(string, tag = "2")]
    pub source_channel: String,

    #[prost(message, tag = "3")]
    pub token: Option<Coin>,

    #[prost(string, tag = "4")]
    pub sender: String,

    #[prost(string, tag = "5")]
    pub receiver: String,

    #[prost(uint64, tag = "7")]
    pub timeout_timestamp: u64,

    #[prost(string, tag = "8")]
    pub memo: String,
}

pub const DEFAULT_TRANSFER_MSG_TYPE_URL: &str = "/ibc.applications.transfer.v1.MsgTransfer";

// TODO: reduce the number of arguments here
#[allow(clippy::too_many_arguments)]
pub fn generate_ics20_stargate_msg(
    type_url: String,
    source_port: String,
    source_channel: String,
    token: Option<Coin>,
    sender: String,
    receiver: String,
    timeout_timestamp_nanos: u64,
    memo: String,
) -> CosmosMsg {
    Stargate {
        type_url,
        value: MsgTransfer {
            source_port,
            source_channel,
            token,
            sender,
            receiver,
            timeout_timestamp: timeout_timestamp_nanos,
            memo,
        }
        .encode_to_vec()
        .into(),
    }
}

/// Sends over CW20 assets to another chain
pub fn generate_cw20_ics20_transfer_msg(
    cw20_contract: String,
    amount: Uint128,
    cw20_ics20_contract: String,
    source_channel: String,
    receiver: String,
    timeout: u64,
) -> StdResult<CosmosMsg> {
    let transfer_msg = wasm_execute(
        cw20_contract,
        &cw20::Cw20ExecuteMsg::Send {
            contract: cw20_ics20_contract,
            amount,
            msg: to_json_binary(&cw20_ics20::msg::TransferMsg {
                channel: source_channel,
                remote_address: receiver,
                timeout: Some(timeout),
                memo: None,
            })?,
        },
        vec![],
    )?;

    Ok(transfer_msg.into())
}
