use crate::api::{
    AllDaosResponse, Config, ConfigResponse, CreateDaoMsg, EnterpriseCodeIdsMsg,
    EnterpriseCodeIdsResponse, IsEnterpriseCodeIdMsg, IsEnterpriseCodeIdResponse, QueryAllDaosMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cw_asset::AssetInfo;
use enterprise_protocol::api::{AssetWhitelistResponse, NftWhitelistResponse};

#[cw_serde]
pub struct InstantiateMsg {
    pub config: Config,
    pub global_asset_whitelist: Option<Vec<AssetInfo>>,
    pub global_nft_whitelist: Option<Vec<Addr>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateDao(CreateDaoMsg),
}

#[cw_serde]
pub struct MigrateMsg {
    pub new_enterprise_code_id: Option<u64>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AssetWhitelistResponse)]
    GlobalAssetWhitelist {},
    #[returns(NftWhitelistResponse)]
    GlobalNftWhitelist {},
    #[returns(AllDaosResponse)]
    AllDaos(QueryAllDaosMsg),
    #[returns(EnterpriseCodeIdsResponse)]
    EnterpriseCodeIds(EnterpriseCodeIdsMsg),
    #[returns(IsEnterpriseCodeIdResponse)]
    IsEnterpriseCodeId(IsEnterpriseCodeIdMsg),
}
