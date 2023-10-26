use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Event};
use cw_asset::AssetInfoUnchecked;

#[cw_serde]
pub struct CrossChainTreasury {
    pub chain_id: String,
    pub treasury_addr: String,
}

#[cw_serde]
pub struct DeployCrossChainTreasuryMsg {
    pub cross_chain_msg_spec: CrossChainMsgSpec,
    pub asset_whitelist: Option<Vec<AssetInfoUnchecked>>,
    pub nft_whitelist: Option<Vec<String>>,
    pub ics_proxy_code_id: u64,
    pub enterprise_treasury_code_id: u64,
    /// Proxy contract serving globally for the given chain, with no specific permission model.
    pub chain_global_proxy: String,
}

#[cw_serde]
pub struct CrossChainMsgSpec {
    pub chain_id: String,
    pub chain_bech32_prefix: String,
    pub src_ibc_port: String,
    pub src_ibc_channel: String,
    pub dest_ibc_port: String,
    pub dest_ibc_channel: String,
    /// uluna IBC denom on the remote chain. Currently, can be calculated as 'ibc/' + uppercase(sha256('{port}/{channel}/uluna'))
    pub uluna_denom: String,
    /// Optional timeout for the cross-chain messages. Formatted in nanoseconds.
    pub timeout_nanos: Option<u64>,
}

#[cw_serde]
pub struct ExecuteMsgReplyCallbackMsg {
    pub callback_id: u32,
    pub events: Vec<Event>,
    pub data: Option<Binary>,
}

#[cw_serde]
pub struct CrossChainTreasuriesParams {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct CrossChainDeploymentsParams {
    pub chain_id: String,
}

// Responses

#[cw_serde]
pub struct CrossChainTreasuriesResponse {
    pub treasuries: Vec<CrossChainTreasury>,
}

#[cw_serde]
pub struct CrossChainDeploymentsResponse {
    pub chain_id: String,
    pub proxy_addr: Option<String>,
    pub treasury_addr: Option<String>,
}
