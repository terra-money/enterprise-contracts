use crate::api::{
    AssetWhitelistParams, AssetWhitelistResponse, ConfigResponse, DistributeFundsMsg,
    ExecuteCosmosMsgsMsg, HasIncompleteV2MigrationResponse, NftWhitelistParams,
    NftWhitelistResponse, SetAdminMsg, SpendMsg, UpdateAssetWhitelistMsg, UpdateNftWhitelistMsg,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_asset::AssetInfoUnchecked;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
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

    /// To be called only when there is an unfinished migration from pre-1.0.0 Enterprise
    PerformNextMigrationStep {
        submsgs_limit: Option<u32>,
    },

    /// Called by self to finalize initial migration step. Not part of the public API!
    FinalizeInitialMigrationStep {},
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

    /// Used to determine whether this contract is still in the middle of migration from
    /// old contracts to new contracts.
    #[returns(HasIncompleteV2MigrationResponse)]
    HasIncompleteV2Migration {},
}

#[cw_serde]
pub struct MigrateMsg {}
