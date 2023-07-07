use crate::api::{
    AssetWhitelistParams, AssetWhitelistResponse, ConfigResponse, DistributeFundsMsg,
    ExecuteCosmosMsgsMsg, NftWhitelistParams, NftWhitelistResponse, SetAdminMsg, SpendMsg,
    UpdateAssetWhitelistMsg, UpdateNftWhitelistMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_asset::AssetInfoUnchecked;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub enterprise_contract: String,
    pub asset_whitelist: Option<Vec<AssetInfoUnchecked>>,
    pub nft_whitelist: Option<Vec<String>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetAdmin(SetAdminMsg),
    UpdateAssetWhitelist(UpdateAssetWhitelistMsg),
    UpdateNftWhitelist(UpdateNftWhitelistMsg),
    Spend(SpendMsg),
    DistributeFunds(DistributeFundsMsg),
    ExecuteCosmosMsgs(ExecuteCosmosMsgsMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AssetWhitelistResponse)]
    AssetWhitelist(AssetWhitelistParams),
    #[returns(NftWhitelistResponse)]
    NftWhitelist(NftWhitelistParams),
}

#[cw_serde]
pub struct MigrateMsg {}
