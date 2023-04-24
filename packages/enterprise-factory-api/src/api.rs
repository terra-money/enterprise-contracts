use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint64};
use cw_asset::AssetInfo;
use enterprise_protocol::api::{
    DaoCouncilSpec, DaoGovConfig, DaoMetadata, ExistingDaoMembershipMsg, NewMembershipInfo,
};

#[cw_serde]
pub struct Config {
    pub enterprise_code_id: u64,
    pub enterprise_governance_code_id: u64,
    pub funds_distributor_code_id: u64,
    pub cw3_fixed_multisig_code_id: u64,
    pub cw20_code_id: u64,
    pub cw721_code_id: u64,
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct CreateDaoMsg {
    pub dao_metadata: DaoMetadata,
    pub dao_gov_config: DaoGovConfig,
    /// Optional council structure that can manage certain aspects of the DAO
    pub dao_council: Option<DaoCouncilSpec>,
    pub dao_membership: CreateDaoMembershipMsg,
    /// assets that are allowed to show in DAO's treasury
    pub asset_whitelist: Option<Vec<AssetInfo>>,
    /// NFTs that are allowed to show in DAO's treasury
    pub nft_whitelist: Option<Vec<Addr>>,
    /// Minimum weight that a user should have in order to qualify for rewards.
    /// E.g. a value of 3 here means that a user in token or NFT DAO needs at least 3 staked
    /// DAO assets, or a weight of 3 in multisig DAO, to be eligible for rewards.
    pub minimum_weight_for_rewards: Option<Uint128>,
}

#[cw_serde]
#[allow(clippy::large_enum_variant)]
pub enum CreateDaoMembershipMsg {
    NewMembership(NewMembershipInfo),
    ExistingMembership(ExistingDaoMembershipMsg),
}

#[cw_serde]
pub struct QueryAllDaosMsg {
    pub start_after: Option<Uint64>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct AllDaosResponse {
    pub daos: Vec<Addr>,
}

#[cw_serde]
pub struct EnterpriseCodeIdsMsg {
    pub start_after: Option<Uint64>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct EnterpriseCodeIdsResponse {
    pub code_ids: Vec<Uint64>,
}

#[cw_serde]
pub struct IsEnterpriseCodeIdMsg {
    pub code_id: Uint64,
}

#[cw_serde]
pub struct IsEnterpriseCodeIdResponse {
    pub is_enterprise_code_id: bool,
}
