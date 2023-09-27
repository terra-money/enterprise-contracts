use cosmwasm_schema::cw_serde;
use cosmwasm_std::CosmosMsg::Stargate;
use cosmwasm_std::{CosmosMsg, Env, SubMsg};
use cw_storage_plus::{Item, Map};
use enterprise_governance_controller_api::api::{CrossChainMsgSpec, DeployCrossChainTreasuryMsg};
use enterprise_governance_controller_api::error::GovernanceControllerResult;
use prost::Message;

use bech32_no_std::ToBase32;
use sha2::{Digest, Sha256};

#[cw_serde]
pub struct IcsProxyInstantiateMsg {
    /// This is a flag that can block this contract from executing cross-chain messages.
    /// Mainly used to prevent fake reports of this contract's callbacks.
    pub allow_cross_chain_msgs: bool,
    pub owner: Option<String>,
    pub whitelist: Option<Vec<String>>,
    pub msgs: Option<Vec<CosmosMsg>>,
}

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
    callback_id: Option<u32>,
) -> GovernanceControllerResult<SubMsg> {
    let reply_callback = callback_id.map(|callback_id| ReplyCallback {
        callback_id,
        ibc_port: cross_chain_msg_spec.dest_ibc_port,
        ibc_channel: cross_chain_msg_spec.dest_ibc_channel,
        denom: cross_chain_msg_spec.uluna_denom,
        receiver: Some(env.contract.address.to_string()),
    });

    let memo = IbcHooksProxyMemoMsg {
        wasm: IbcHooksProxyWasmMsg {
            contract: proxy_contract.clone(),
            msg: IcsProxyExecuteMsg::ExecuteMsgs(ExecuteMsgsMsg {
                msgs: vec![ExecuteMsgInfo {
                    msg,
                    reply_callback,
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
pub const ICS_PROXY_CALLBACKS: Map<u32, IcsProxyCallback> = Map::new("ics_proxy_callbacks");

pub const ICS_PROXY_CALLBACK_LAST_ID: Item<u32> = Item::new("ics_proxy_callback_last_id");

#[cw_serde]
// TODO: write an explanation
pub struct IcsProxyCallback {
    pub cross_chain_msg_spec: CrossChainMsgSpec,
    /// Address of the proxy, in its own native chain representation. E.g. proxy on juno would be 'juno1ahe3aw...'
    pub proxy_addr: String,
    pub callback_type: IcsProxyCallbackType,
}

#[cw_serde]
pub enum IcsProxyCallbackType {
    InstantiateProxy {
        deploy_treasury_msg: Box<DeployCrossChainTreasuryMsg>,
    },
    InstantiateTreasury {
        cross_chain_msg_spec: CrossChainMsgSpec,
    },
}

/// Prefix for Bech32 addresses on Terra. E.g. 'terra1y2dwydn...'
pub const TERRA_CHAIN_BECH32_PREFIX: &str = "terra";

const SENDER_PREFIX: &str = "ibc-wasm-hook-intermediary";

/// Derives the sender address that will be used instead of the original sender's address
/// when using IBC hooks cross-chain.
/// ```rust
/// use enterprise_governance_controller::ibc_hooks::derive_intermediate_sender;
/// let original_sender =   "juno12smx2wdlyttvyzvzg54y2vnqwq2qjatezqwqxu";
/// let hashed_sender = derive_intermediate_sender("channel-0", original_sender, "osmo").unwrap();
/// assert_eq!(hashed_sender, "osmo1nt0pudh879m6enw4j6z4mvyu3vmwawjv5gr7xw6lvhdsdpn3m0qs74xdjl");
/// ```
pub fn derive_intermediate_sender(
    channel: &str,
    original_sender: &str,
    bech32_prefix: &str,
) -> Result<String, bech32_no_std::Error> {
    let sender_path = format!("{channel}/{original_sender}");

    let sender_hash_32 = prefixed_sha256(SENDER_PREFIX, &sender_path);

    bech32_no_std::encode(bech32_prefix, sender_hash_32.to_base32())
}

pub fn prefixed_sha256(prefix: &str, address: &str) -> [u8; 32] {
    let mut hasher = Sha256::default();

    hasher.update(prefix.as_bytes());
    let prefix_hash = hasher.finalize();

    let mut hasher = Sha256::default();

    hasher.update(prefix_hash);
    hasher.update(address.as_bytes());

    hasher.finalize().into()
}
