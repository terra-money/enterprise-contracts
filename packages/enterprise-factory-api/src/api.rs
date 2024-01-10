use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint64};
use cw20::{Cw20Coin, MinterResponse};
use cw_asset::AssetInfoUnchecked;
use cw_utils::Duration;
use enterprise_governance_controller_api::api::{DaoCouncilSpec, GovConfig};
use enterprise_outposts_api::api::DeployCrossChainTreasuryMsg;
use enterprise_protocol::api::DaoMetadata;
use multisig_membership_api::api::UserWeight;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub enterprise_versioning: Addr,
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
    pub gov_config: GovConfig,
    /// Optional council structure that can manage certain aspects of the DAO
    pub dao_council: Option<DaoCouncilSpec>,
    pub dao_membership: CreateDaoMembershipMsg,
    /// assets that are allowed to show in DAO's treasury
    pub asset_whitelist: Option<Vec<AssetInfoUnchecked>>,
    /// NFTs that are allowed to show in DAO's treasury
    pub nft_whitelist: Option<Vec<String>>,
    /// Minimum weight that a user should have in order to qualify for rewards.
    /// E.g. a value of 3 here means that a user in token or NFT DAO needs at least 3 staked
    /// DAO assets, or a weight of 3 in multisig DAO, to be eligible for rewards.
    pub minimum_weight_for_rewards: Option<Uint128>,
    /// Optional cross chain treasuries to deploy during DAO creation.
    pub cross_chain_treasuries: Option<Vec<DeployCrossChainTreasuryMsg>>,
    /// Optional text that users will have to attest to before being able to participate in DAO's
    /// governance and certain other functions.
    pub attestation_text: Option<String>,
}

#[cw_serde]
pub struct UpdateConfigMsg {
    pub new_admin: Option<String>,
    pub new_enterprise_versioning: Option<String>,
    pub new_cw20_code_id: Option<u64>,
    pub new_cw721_code_id: Option<u64>,
}

#[cw_serde]
pub enum CreateDaoMembershipMsg {
    NewDenom(NewDenomMembershipMsg),
    ImportCw20(ImportCw20MembershipMsg),
    NewCw20(Box<NewCw20MembershipMsg>),
    ImportCw721(ImportCw721MembershipMsg),
    NewCw721(NewCw721MembershipMsg),
    ImportCw3(ImportCw3MembershipMsg),
    NewMultisig(NewMultisigMembershipMsg),
}

#[cw_serde]
pub struct ImportCw20MembershipMsg {
    /// Address of the CW20 token to import
    pub cw20_contract: String,
    /// Duration after which unstaked tokens can be claimed
    pub unlocking_period: Duration,
}

#[cw_serde]
pub struct NewCw20MembershipMsg {
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimals: u8,
    pub initial_token_balances: Vec<Cw20Coin>,
    /// Optional amount of tokens to be minted to the DAO's address
    pub initial_dao_balance: Option<Uint128>,
    pub token_mint: Option<MinterResponse>,
    pub token_marketing: Option<TokenMarketingInfo>,
    pub unlocking_period: Duration,
}

#[cw_serde]
pub struct NewDenomMembershipMsg {
    pub denom: String,
    pub unlocking_period: Duration,
}

#[cw_serde]
pub struct TokenMarketingInfo {
    pub project: Option<String>,
    pub description: Option<String>,
    pub marketing_owner: Option<String>,
    pub logo_url: Option<String>,
}

#[cw_serde]
pub struct ImportCw721MembershipMsg {
    /// Address of the CW721 contract to import
    pub cw721_contract: String,
    /// Duration after which unstaked items can be claimed
    pub unlocking_period: Duration,
}

#[cw_serde]
pub struct NewCw721MembershipMsg {
    pub nft_name: String,
    pub nft_symbol: String,
    pub minter: Option<String>,
    pub unlocking_period: Duration,
}

#[cw_serde]
pub struct ImportCw3MembershipMsg {
    /// Address of the CW3 contract to import
    pub cw3_contract: String,
}

#[cw_serde]
pub struct NewMultisigMembershipMsg {
    pub multisig_members: Vec<UserWeight>,
}

#[cw_serde]
pub struct QueryAllDaosMsg {
    pub start_after: Option<Uint64>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct AllDaosResponse {
    pub daos: Vec<DaoRecord>,
}

#[cw_serde]
pub struct DaoRecord {
    pub dao_id: Uint64,
    pub dao_address: Addr,
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
