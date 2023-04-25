use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Timestamp, Uint128, Uint64};

pub type NftTokenId = String;

// TODO: move to common and remove other declarations of this
#[cw_serde]
pub enum ReleaseAt {
    Timestamp(Timestamp),
    Height(Uint64),
}

// TODO: also move to common
#[cw_serde]
pub struct ReceiveNftMsg {
    // this exists so we're Talis-compatible, otherwise it's not part of the CW721 standard
    pub edition: Option<Uint64>,
    pub sender: String,
    pub token_id: String,
    pub msg: Binary,
}

#[cw_serde]
pub struct UnstakeMsg {
    pub user: String,
    pub nft_ids: Vec<NftTokenId>,
}

#[cw_serde]
pub struct ClaimMsg {
    pub user: String,
}

#[cw_serde]
pub struct UpdateAdminMsg {
    pub new_admin: String,
}

#[cw_serde]
pub struct UserNftStakeParams {
    pub user: String,
    pub start_after: Option<NftTokenId>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct ClaimsParams {
    pub user: String,
}

#[cw_serde]
pub struct NftClaim {
    pub user: Addr,
    pub nfts: Vec<NftTokenId>,
    pub release_at: ReleaseAt,
}

////// Responses

#[cw_serde]
pub struct UserNftStakeResponse {
    pub user: Addr,
    pub tokens: Vec<NftTokenId>,
    pub amount: Uint128,
}

#[cw_serde]
pub struct ClaimsResponse {
    pub claims: Vec<NftClaim>,
}

#[cw_serde]
pub struct TotalStakedAmountResponse {
    pub total_staked_amount: Uint128,
}
