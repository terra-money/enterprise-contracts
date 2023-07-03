use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_asset::{Asset, AssetInfo};

#[cw_serde]
pub struct UpdateConfigMsg {
    pub new_admin: String,
}

#[cw_serde]
pub struct UpdateAssetWhitelistMsg {
    /// New assets to add to the whitelist. Will ignore assets that are already whitelisted.
    pub add: Vec<AssetInfo>,
    /// Assets to remove from the whitelist. Will ignore assets that are not already whitelisted.
    pub remove: Vec<AssetInfo>,
}

#[cw_serde]
pub struct UpdateNftWhitelistMsg {
    /// New NFTs to add to the whitelist. Will ignore NFTs that are already whitelisted.
    pub add: Vec<String>,
    /// NFTs to remove from the whitelist. Will ignore NFTs that are not already whitelisted.
    pub remove: Vec<Addr>,
}

#[cw_serde]
pub struct SpendMsg {
    pub recipient: String,
    pub assets: Vec<Asset>,
}

#[cw_serde]
pub struct DistributeFundsMsg {
    pub funds: Vec<Asset>,
}

#[cw_serde]
pub struct ExecuteCosmosMsgsMsg {
    /// custom Cosmos msgs to execute
    pub msgs: Vec<String>,
}

#[cw_serde]
pub struct AssetWhitelistParams {
    pub start_after: Option<AssetInfo>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct NftWhitelistParams {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

////// Responses

#[cw_serde]
pub struct ConfigResponse {
    pub admin: Addr,
}

#[cw_serde]
pub struct AssetWhitelistResponse {
    pub assets: Vec<AssetInfo>,
}

#[cw_serde]
pub struct NftWhitelistResponse {
    pub nfts: Vec<Addr>,
}
