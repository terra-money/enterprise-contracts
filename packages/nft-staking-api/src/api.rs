use common::cw::ReleaseAt;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint64};

pub type NftTokenId = String;

#[cw_serde]
pub struct UnstakeMsg {
    pub user: String,
    pub nft_ids: Vec<NftTokenId>,
}

#[cw_serde]
pub struct UserNftStakeParams {
    pub user: String,
    pub start_after: Option<NftTokenId>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct NftClaim {
    pub id: Uint64,
    pub user: Addr,
    pub nft_ids: Vec<NftTokenId>,
    pub release_at: ReleaseAt,
}

////// Responses

#[cw_serde]
pub struct UserNftStakeResponse {
    pub user: Addr,
    pub tokens: Vec<NftTokenId>,
    pub total_user_stake: Uint128,
}

#[cw_serde]
pub struct UserNftTotalStakeResponse {
    pub user: Addr,
    pub total_user_stake: Uint128,
}

#[cw_serde]
pub struct ClaimsResponse {
    pub claims: Vec<NftClaim>,
}
