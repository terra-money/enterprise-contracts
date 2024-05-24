use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;

use crate::token_types::{ClassId, TokenId};

#[cw_serde]
#[serde(rename_all = "camelCase")]
pub struct NonFungibleTokenPacketData {
    /// Uniquely identifies the collection which the tokens being
    /// transfered belong to on the sending chain. Must be non-empty.
    pub class_id: ClassId,
    /// Optional URL that points to metadata about the
    /// collection. Must be non-empty if provided.
    pub class_uri: Option<String>,
    /// Optional base64 encoded field which contains on-chain metadata
    /// about the NFT class. Must be non-empty if provided.
    pub class_data: Option<Binary>,
    /// Uniquely identifies the tokens in the NFT collection being
    /// transfered. This MUST be non-empty.
    pub token_ids: Vec<TokenId>,
    /// Optional URL that points to metadata for each token being
    /// transfered. `tokenUris[N]` should hold the metadata for
    /// `tokenIds[N]` and both lists should have the same if
    /// provided. Must be non-empty if provided.
    pub token_uris: Option<Vec<String>>,
    /// Optional base64 encoded metadata for the tokens being
    /// transfered. `tokenData[N]` should hold metadata for
    /// `tokenIds[N]` and both lists should have the same length if
    /// provided. Must be non-empty if provided.
    pub token_data: Option<Vec<Binary>>,

    /// The address sending the tokens on the sending chain.
    pub sender: String,
    /// The address that should receive the tokens on the receiving
    /// chain.
    pub receiver: String,
    /// Memo to add custom string to the msg
    pub memo: Option<String>,
}
