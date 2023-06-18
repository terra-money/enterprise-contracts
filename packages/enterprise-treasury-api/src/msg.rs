use crate::api::{
    AssetWhitelistParams, AssetWhitelistResponse, ConfigResponse, DistributeFundsMsg,
    ExecuteCosmosMsgsMsg, NftWhitelistParams, NftWhitelistResponse, SpendMsg,
    UpdateAssetWhitelistMsg, UpdateConfigMsg, UpdateNftWhitelistMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_asset::AssetInfo;

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_contract: String,
    pub asset_whitelist: Option<Vec<AssetInfo>>,
    pub nft_whitelist: Option<Vec<String>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig(UpdateConfigMsg),
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