use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Decimal, Timestamp, Uint128, Uint64};
use cw20::{Cw20Coin, MinterResponse};
use cw_asset::{Asset, AssetInfo};
use cw_utils::Duration;
use std::fmt;
use strum_macros::Display;

pub type ProposalId = u64;
pub type NftTokenId = String;

#[cw_serde]
pub enum ModifyValue<T> {
    Change(T),
    NoChange,
}

#[cw_serde]
#[derive(Display)]
pub enum DaoType {
    Token,
    Nft,
    Multisig,
}

#[cw_serde]
pub struct DaoMetadata {
    pub name: String,
    pub description: Option<String>,
    pub logo: Logo,
    pub socials: DaoSocialData,
}

#[cw_serde]
pub enum Logo {
    // TODO: think about allowing on-chain logo
    Url(String),
    None,
}

impl fmt::Display for Logo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Logo::Url(url) => write!(f, "url: {}", url),
            Logo::None => write!(f, "none"),
        }
    }
}

#[cw_serde]
pub struct DaoSocialData {
    pub github_username: Option<String>,
    pub discord_username: Option<String>,
    pub twitter_username: Option<String>,
    pub telegram_username: Option<String>,
}

#[cw_serde]
pub enum DaoMembershipInfo {
    New(NewDaoMembershipMsg),
    Existing(ExistingDaoMembershipMsg),
}

#[cw_serde]
pub struct NewDaoMembershipMsg {
    pub membership_contract_code_id: u64,
    pub membership_info: NewMembershipInfo,
}

#[cw_serde]
pub enum NewMembershipInfo {
    NewToken(Box<NewTokenMembershipInfo>),
    NewNft(NewNftMembershipInfo),
    NewMultisig(NewMultisigMembershipInfo),
}

#[cw_serde]
pub struct ExistingDaoMembershipMsg {
    pub dao_type: DaoType,
    pub membership_contract_addr: String,
}

#[cw_serde]
pub struct NewTokenMembershipInfo {
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimals: u8,
    pub initial_token_balances: Vec<Cw20Coin>,
    /// Optional amount of tokens to be minted to the DAO's address
    pub initial_dao_balance: Option<Uint128>,
    pub token_mint: Option<MinterResponse>,
    pub token_marketing: Option<TokenMarketingInfo>,
}

#[cw_serde]
pub struct TokenMarketingInfo {
    pub project: Option<String>,
    pub description: Option<String>,
    pub marketing_owner: Option<String>,
    pub logo_url: Option<String>,
}

#[cw_serde]
pub struct NewNftMembershipInfo {
    pub nft_name: String,
    pub nft_symbol: String,
    pub minter: Option<String>,
}

#[cw_serde]
pub struct NewMultisigMembershipInfo {
    pub multisig_members: Vec<MultisigMember>,
}

#[cw_serde]
pub struct MultisigMember {
    pub address: String,
    pub weight: Uint128,
}

#[cw_serde]
pub struct UpdateMetadataMsg {
    pub name: ModifyValue<String>,
    pub description: ModifyValue<Option<String>>,
    pub logo: ModifyValue<Logo>,
    pub github_username: ModifyValue<Option<String>>,
    pub discord_username: ModifyValue<Option<String>>,
    pub twitter_username: ModifyValue<Option<String>>,
    pub telegram_username: ModifyValue<Option<String>>,
}

#[cw_serde]
pub struct UpdateGovConfigMsg {
    pub quorum: ModifyValue<Decimal>,
    pub threshold: ModifyValue<Decimal>,
    pub veto_threshold: ModifyValue<Option<Decimal>>,
    pub voting_duration: ModifyValue<Uint64>,
    pub unlocking_period: ModifyValue<Duration>,
    pub minimum_deposit: ModifyValue<Option<Uint128>>,
    pub allow_early_proposal_execution: ModifyValue<bool>,
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
    pub add: Vec<Addr>,
    /// NFTs to remove from the whitelist. Will ignore NFTs that are not already whitelisted.
    pub remove: Vec<Addr>,
}

#[cw_serde]
pub struct RequestFundingFromDaoMsg {
    pub recipient: String,
    pub assets: Vec<Asset>,
}

#[cw_serde]
pub struct UpgradeDaoMsg {
    pub new_dao_code_id: u64,
    pub migrate_msg: Binary,
}

#[cw_serde]
pub struct ExecuteMsgsMsg {
    pub action_type: String,
    pub msgs: Vec<String>,
}

#[cw_serde]
pub struct ModifyMultisigMembershipMsg {
    /// Members to be edited.
    /// Can contain existing members, in which case their new weight will be the one specified in
    /// this message. This effectively allows removing of members (by setting their weight to 0).
    pub edit_members: Vec<MultisigMember>,
}

#[cw_serde]
pub struct DistributeFundsMsg {
    pub funds: Vec<Asset>,
}

#[cw_serde]
pub struct DaoInfoResponse {
    pub creation_date: Timestamp,
    pub metadata: DaoMetadata,
    pub dao_type: DaoType,
    pub dao_membership_contract: Addr,
    pub enterprise_factory_contract: Addr,
    pub funds_distributor_contract: Addr,
    pub dao_code_version: Uint64,
}

#[cw_serde]
pub struct AssetWhitelistParams {
    pub start_after: Option<AssetInfo>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct AssetWhitelistResponse {
    pub assets: Vec<AssetInfo>,
}

#[cw_serde]
pub struct NftWhitelistParams {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct NftWhitelistResponse {
    pub nfts: Vec<Addr>,
}
