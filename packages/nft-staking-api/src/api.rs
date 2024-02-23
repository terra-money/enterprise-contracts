use common::cw::ReleaseAt;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Uint128, Uint64};
use cw_utils::Duration;

pub type NftTokenId = String;

#[cw_serde]
pub enum NftContract {
    Cw721 { contract: String },
    Ics721 { contract: String, class_id: String },
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
    pub nft_ids: Vec<NftTokenId>,
}

#[cw_serde]
pub struct ClaimMsg {
    pub user: Option<String>,
}

#[cw_serde]
pub struct UpdateUnlockingPeriodMsg {
    pub new_unlocking_period: Option<Duration>,
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
    pub id: Uint64,
    pub user: Addr,
    pub nft_ids: Vec<NftTokenId>,
    pub release_at: ReleaseAt,
}

#[cw_serde]
pub struct StakedNftsParams {
    pub start_after: Option<NftTokenId>,
    pub limit: Option<u32>,
}

////// Responses

#[cw_serde]
pub struct NftConfigResponse {
    pub enterprise_contract: Addr,
    pub nft_contract: Addr,
    pub unlocking_period: Duration,
}

#[cw_serde]
pub struct Ics721ConfigResponse {
    pub enterprise_contract: Addr,
    pub ics721_contract: Addr,
    pub class_id: String,
    pub unlocking_period: Duration,
}

#[cw_serde]
pub struct UserNftStakeResponse {
    pub user: Addr,
    pub tokens: Vec<NftTokenId>,
    pub total_user_stake: Uint128,
}

#[cw_serde]
pub struct ClaimsResponse {
    pub claims: Vec<NftClaim>,
}

#[cw_serde]
pub struct StakedNftsResponse {
    pub nfts: Vec<NftTokenId>,
}
