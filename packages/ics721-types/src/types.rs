use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;

use crate::ibc_types::NonFungibleTokenPacketData;

/// A message is that is being called on receiving the NFT after transfer was completed.
/// Receiving this message means that the NFT was successfully transferred.
/// You must verify this message was called by an approved ICS721 contract, either by code_id or address.
#[cw_serde]
pub struct Ics721ReceiveCallbackMsg {
    pub nft_contract: String,
    pub original_packet: NonFungibleTokenPacketData,
    pub msg: Binary,
}
