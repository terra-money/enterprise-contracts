use cosmwasm_std::CosmosMsg;
use cosmwasm_std::CosmosMsg::Stargate;
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
    timeout_timestamp: u64,
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
            timeout_timestamp,
            memo,
        }
        .encode_to_vec()
        .into(),
    }
}
