use cosmwasm_std::{Binary, Empty};
use cw721::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug, Default)]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

// see: https://docs.opensea.io/docs/metadata-standards
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug, Default)]
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// cw721 Transfer NFT message
pub struct TransferNftMsg {
    pub recipient: String,
    pub token_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// cw721 Send NFT message
pub struct SendNftMsg {
    pub contract: String,
    pub token_id: String,
    pub msg: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// cw721 Approve token usage message
pub struct ApproveMsg {
    pub spender: String,
    pub token_id: String,
    pub expires: Option<Expiration>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// cw721 Revoke Approval message
pub struct RevokeMsg {
    pub spender: String,
    pub token_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// cw721 Approve all tokens message
pub struct ApproveAllMsg {
    pub operator: String,
    pub expires: Option<Expiration>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// cw721 Revoke all approval message
pub struct RevokeAllMsg {
    pub operator: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// cw721 Query Owner of token message
pub struct QueryOwnerOfMsg {
    pub token_id: String,
    pub include_expired: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// cw721 Query approval status message
pub struct QueryApprovalMsg {
    pub token_id: String,
    pub spender: String,
    pub include_expired: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// cw721 Query all approvals message
pub struct QueryApprovalsMsg {
    pub token_id: String,
    pub include_expired: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// cw721 Query operators message
pub struct QueryAllOperatorsMsg {
    pub owner: String,
    pub include_expired: Option<bool>,
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// cw721 Query number of tokens message
pub struct QueryNumTokensMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// cw721 Query contract info message
pub struct QueryContractInfoMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// cw721 Query nft info message
pub struct QueryNftInfoMsg {
    pub token_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// cw721 Query all nft info message
pub struct QueryAllNftInfoMsg {
    pub token_id: String,
    pub include_expired: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// cw721 Query tokens message
pub struct QueryTokensMsg {
    pub owner: String,
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// cw721 Query all tokens message
pub struct QueryAllTokensMsg {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

pub type Extension = Option<Metadata>;
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension, Empty>;
pub type MintMsg = cw721_base::MintMsg<Extension>;
pub use cw721::TokensResponse;
pub use cw721_base::{ContractError, InstantiateMsg, MinterResponse, QueryMsg};
