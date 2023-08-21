use cosmwasm_schema::cw_serde;
use cosmwasm_std::CosmosMsg::Stargate;
use cosmwasm_std::{CosmosMsg, Env, SubMsg};
use cw_storage_plus::Map;
use enterprise_governance_controller_api::api::CrossChainMsgSpec;
use enterprise_governance_controller_api::error::GovernanceControllerResult;
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

#[cw_serde]
pub struct IbcHooksProxyMemoMsg {
    pub wasm: IbcHooksProxyWasmMsg,
}

#[cw_serde]
pub struct IbcHooksProxyWasmMsg {
    pub contract: String,
    pub msg: IcsProxyExecuteMsg,
}

#[cw_serde]
pub enum IcsProxyExecuteMsg {
    ExecuteMsgs(ExecuteMsgsMsg),
}

#[cw_serde]
pub struct ExecuteMsgsMsg {
    pub msgs: Vec<ExecuteMsgInfo>,
}

#[cw_serde]
pub struct ExecuteMsgInfo {
    pub msg: CosmosMsg,
    pub reply_callback: Option<ReplyCallback>,
}

#[cw_serde]
pub struct ReplyCallback {
    pub callback_id: u32,
    pub ibc_port: String,
    pub ibc_channel: String,
    // denom to send back when replying
    pub denom: String,
    pub receiver: Option<String>,
}

pub fn ibc_hooks_msg_to_ics_proxy_contract(
    env: &Env,
    msg: CosmosMsg,
    proxy_contract: String,
    cross_chain_msg_spec: CrossChainMsgSpec,
    callback_id: u32,
) -> GovernanceControllerResult<SubMsg> {
    let memo = IbcHooksProxyMemoMsg {
        wasm: IbcHooksProxyWasmMsg {
            contract: proxy_contract.clone(),
            msg: IcsProxyExecuteMsg::ExecuteMsgs(ExecuteMsgsMsg {
                msgs: vec![ExecuteMsgInfo {
                    msg,
                    reply_callback: Some(ReplyCallback {
                        callback_id,
                        ibc_port: cross_chain_msg_spec.dest_ibc_port,
                        ibc_channel: cross_chain_msg_spec.dest_ibc_channel,
                        denom: cross_chain_msg_spec.uluna_denom,
                        receiver: Some(env.contract.address.to_string()),
                    }),
                }],
            }),
        },
    };
    let stargate_msg = Stargate {
        type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
        value: MsgTransfer {
            source_port: cross_chain_msg_spec.src_ibc_port,
            source_channel: cross_chain_msg_spec.src_ibc_channel,
            token: Some(Coin {
                denom: "uluna".to_string(),
                amount: "1".to_string(),
            }),
            sender: env.contract.address.to_string(),
            receiver: proxy_contract,
            timeout_timestamp: cross_chain_msg_spec
                .timeout_nanos
                .unwrap_or_else(|| env.block.time.plus_hours(1).nanos()),
            memo: serde_json_wasm::to_string(&memo)?,
        }
        .encode_to_vec()
        .into(),
    };

    Ok(SubMsg::new(stargate_msg))
}

/// A map of ICS proxy contract callbacks we're expecting.
/// Will store a pair of callback ID and proxy contract address.
pub const ICS_PROXY_CALLBACKS: Map<(u32, String), ()> = Map::new("ics_proxy_callbacks");
