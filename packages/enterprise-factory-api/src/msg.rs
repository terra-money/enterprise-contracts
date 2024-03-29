use crate::api::{
    AllDaosResponse, Config, ConfigResponse, CreateDaoMsg, CreateDaoWithVersionMsg,
    EnterpriseCodeIdsMsg, EnterpriseCodeIdsResponse, IsEnterpriseCodeIdMsg,
    IsEnterpriseCodeIdResponse, QueryAllDaosMsg, UpdateConfigMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_asset::AssetInfoUnchecked;
use enterprise_treasury_api::api::{AssetWhitelistResponse, NftWhitelistResponse};

#[cw_serde]
pub struct InstantiateMsg {
    pub config: Config,
    pub global_asset_whitelist: Option<Vec<AssetInfoUnchecked>>,
    pub global_nft_whitelist: Option<Vec<String>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateDao(Box<CreateDaoMsg>),

    CreateDaoWithVersion(Box<CreateDaoWithVersionMsg>),

    /// Admin only can execute
    UpdateConfig(UpdateConfigMsg),

    /// Executed only by this contract itself to finalize creation of a DAO.
    /// Not part of the public interface.
    FinalizeDaoCreation {},
}

#[cw_serde]
pub struct MigrateMsg {
    pub admin: String,
    pub enterprise_versioning_addr: String,
    pub cw20_code_id: Option<u64>,
    pub cw721_code_id: Option<u64>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AllDaosResponse)]
    AllDaos(QueryAllDaosMsg),
    #[returns(EnterpriseCodeIdsResponse)]
    EnterpriseCodeIds(EnterpriseCodeIdsMsg),
    #[returns(IsEnterpriseCodeIdResponse)]
    IsEnterpriseCodeId(IsEnterpriseCodeIdMsg),
    #[returns(AssetWhitelistResponse)]
    GlobalAssetWhitelist {},
    #[returns(NftWhitelistResponse)]
    GlobalNftWhitelist {},
}
