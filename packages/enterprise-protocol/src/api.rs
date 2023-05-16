use common::cw::ReleaseAt;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Decimal, StdError, StdResult, Timestamp, Uint128, Uint64};
use cw20::{Cw20Coin, MinterResponse};
use cw721::TokensResponse;
use cw_asset::{Asset, AssetInfo};
use cw_utils::{Duration, Expiration};
use poll_engine_api::api::{Vote, VoteOutcome};
use serde_with::serde_as;
use std::collections::BTreeMap;
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
pub struct DaoGovConfig {
    /// Portion of total available votes cast in a proposal to consider it valid
    /// e.g. quorum of 30% means that 30% of all available votes have to be cast in the proposal,
    /// otherwise it fails automatically when it expires
    pub quorum: Decimal,
    /// Portion of votes assigned to a single option from all the votes cast in the given proposal
    /// required to determine the 'winning' option
    /// e.g. 51% threshold means that an option has to have at least 51% of the cast votes to win
    pub threshold: Decimal,
    /// Portion of votes assigned to veto option from all the votes cast in the given proposal
    /// required to veto the proposal.
    /// If None, will default to the threshold set for all proposal options.
    pub veto_threshold: Option<Decimal>,
    /// Duration of proposals before they end, expressed in seconds
    pub vote_duration: u64, // TODO: change from u64 to Duration
    /// Duration that has to pass for unstaked membership tokens to be claimable
    pub unlocking_period: Duration,
    /// Optional minimum amount of DAO's governance unit to be required to create a deposit.
    pub minimum_deposit: Option<Uint128>,
    /// If set to true, this will allow DAOs to execute proposals that have reached quorum and
    /// threshold, even before their voting period ends.
    pub allow_early_proposal_execution: bool,
}

#[cw_serde]
pub struct DaoCouncilSpec {
    /// Addresses of council members. Each member has equal voting power.
    pub members: Vec<String>,
    /// Portion of total available votes cast in a proposal to consider it valid
    /// e.g. quorum of 30% means that 30% of all available votes have to be cast in the proposal,
    /// otherwise it fails automatically when it expires
    pub quorum: Decimal,
    /// Portion of votes assigned to a single option from all the votes cast in the given proposal
    /// required to determine the 'winning' option
    /// e.g. 51% threshold means that an option has to have at least 51% of the cast votes to win
    pub threshold: Decimal,
    /// Proposal action types allowed in proposals that are voted on by the council.
    /// Effectively defines what types of actions council can propose and vote on.
    /// If None, will default to a predefined set of actions.
    pub allowed_proposal_action_types: Option<Vec<ProposalActionType>>,
}

#[cw_serde]
pub struct DaoCouncil {
    pub members: Vec<Addr>,
    pub allowed_proposal_action_types: Vec<ProposalActionType>,
    pub quorum: Decimal,
    pub threshold: Decimal,
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
pub struct CreateProposalMsg {
    /// Title of the proposal
    pub title: String,
    /// Optional description text of the proposal
    pub description: Option<String>,
    /// Actions to be executed, in order, if the proposal passes
    pub proposal_actions: Vec<ProposalAction>,
}

// TODO: move to poll-engine, together with the deposit returning logic?
#[cw_serde]
pub struct ProposalDeposit {
    pub depositor: Addr,
    pub amount: Uint128,
}

// TODO: try to find a (Rust) language construct allowing us to merge this with ProposalAction
#[cw_serde]
#[derive(Display)]
pub enum ProposalActionType {
    UpdateMetadata,
    UpdateGovConfig,
    UpdateCouncil,
    UpdateAssetWhitelist,
    UpdateNftWhitelist,
    RequestFundingFromDao,
    UpgradeDao,
    ExecuteMsgs,
    ModifyMultisigMembership,
    DistributeFunds,
    UpdateMinimumWeightForRewards,
}

#[cw_serde]
pub enum ProposalAction {
    UpdateMetadata(UpdateMetadataMsg),
    UpdateGovConfig(UpdateGovConfigMsg),
    UpdateCouncil(UpdateCouncilMsg),
    UpdateAssetWhitelist(UpdateAssetWhitelistMsg),
    UpdateNftWhitelist(UpdateNftWhitelistMsg),
    RequestFundingFromDao(RequestFundingFromDaoMsg),
    UpgradeDao(UpgradeDaoMsg),
    ExecuteMsgs(ExecuteMsgsMsg),
    ModifyMultisigMembership(ModifyMultisigMembershipMsg),
    DistributeFunds(DistributeFundsMsg),
    UpdateMinimumWeightForRewards(UpdateMinimumWeightForRewardsMsg),
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
pub struct UpdateCouncilMsg {
    pub dao_council: Option<DaoCouncilSpec>,
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
pub struct UpdateMinimumWeightForRewardsMsg {
    pub minimum_weight_for_rewards: Uint128,
}

#[cw_serde]
pub struct CastVoteMsg {
    pub proposal_id: ProposalId,
    pub outcome: VoteOutcome,
}

#[cw_serde]
pub struct ExecuteProposalMsg {
    pub proposal_id: ProposalId,
}

#[cw_serde]
pub enum UnstakeMsg {
    Cw20(UnstakeCw20Msg),
    Cw721(UnstakeCw721Msg),
}

#[cw_serde]
pub struct UnstakeCw20Msg {
    pub amount: Uint128,
}

#[cw_serde]
pub struct UnstakeCw721Msg {
    pub tokens: Vec<NftTokenId>,
}

#[cw_serde]
pub struct Claim {
    pub asset: ClaimAsset,
    pub release_at: ReleaseAt,
}

#[cw_serde]
pub enum ClaimAsset {
    Cw20(Cw20ClaimAsset),
    Cw721(Cw721ClaimAsset),
}

#[cw_serde]
pub struct Cw20ClaimAsset {
    pub amount: Uint128,
}

#[cw_serde]
pub struct Cw721ClaimAsset {
    pub tokens: Vec<NftTokenId>,
}

#[cw_serde]
pub struct QueryMemberInfoMsg {
    pub member_address: String,
}

#[cw_serde]
pub struct ListMultisigMembersMsg {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct MultisigMembersResponse {
    pub members: Vec<MultisigMember>,
}

#[cw_serde]
pub struct DaoInfoResponse {
    pub creation_date: Timestamp,
    pub metadata: DaoMetadata,
    pub gov_config: DaoGovConfig,
    pub dao_council: Option<DaoCouncil>,
    pub dao_type: DaoType,
    pub dao_membership_contract: Addr,
    pub enterprise_factory_contract: Addr,
    pub funds_distributor_contract: Addr,
    pub staking_contract: Option<Addr>,
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

#[cw_serde]
pub struct MemberInfoResponse {
    pub voting_power: Decimal,
}

#[serde_as]
#[cw_serde]
pub struct ProposalResponse {
    pub proposal: Proposal,

    pub proposal_status: ProposalStatus,

    #[schemars(with = "Vec<(u8, Uint128)>")]
    #[serde_as(as = "Vec<(_, _)>")]
    /// Total vote-count (value) for each outcome (key).
    pub results: BTreeMap<u8, u128>,

    pub total_votes_available: Uint128,
}

#[cw_serde]
pub struct ProposalParams {
    pub proposal_id: ProposalId,
}

#[cw_serde]
pub struct ProposalsResponse {
    pub proposals: Vec<ProposalResponse>,
}

#[cw_serde]
pub struct ProposalsParams {
    /// Optional proposal status to filter for.
    pub filter: Option<ProposalStatusFilter>,
    pub start_after: Option<ProposalId>,
    pub limit: Option<u32>,
    // TODO: allow ordering
}

#[serde_as]
#[cw_serde]
pub struct ProposalStatusResponse {
    pub status: ProposalStatus,
    pub expires: Expiration,

    #[schemars(with = "Vec<(u8, Uint128)>")]
    #[serde_as(as = "Vec<(_, _)>")]
    /// Total vote-count (value) for each outcome (key).
    pub results: BTreeMap<u8, u128>,
}

#[cw_serde]
pub enum ProposalStatus {
    InProgress,
    Passed,
    Rejected,
    Executed,
}

#[cw_serde]
pub enum ProposalStatusFilter {
    InProgress,
    Passed,
    Rejected,
}

impl ProposalStatusFilter {
    pub fn matches(&self, status: &ProposalStatus) -> bool {
        match self {
            ProposalStatusFilter::InProgress => status == &ProposalStatus::InProgress,
            ProposalStatusFilter::Passed => status == &ProposalStatus::Passed,
            ProposalStatusFilter::Rejected => status == &ProposalStatus::Rejected,
        }
    }
}

#[cw_serde]
pub struct ProposalStatusParams {
    pub proposal_id: ProposalId,
}

#[cw_serde]
pub struct MemberVoteParams {
    pub member: String,
    pub proposal_id: ProposalId,
}

#[cw_serde]
pub struct MemberVoteResponse {
    pub vote: Option<Vote>,
}

#[cw_serde]
pub struct ProposalVotesParams {
    pub proposal_id: ProposalId,
    /// Optional pagination data, will return votes after the given voter address
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct ProposalVotesResponse {
    pub votes: Vec<Vote>,
}

#[cw_serde]
pub struct ProposalVotersParams {
    pub proposal_id: ProposalId,
}

#[derive(Display)]
#[cw_serde]
pub enum ProposalType {
    General,
    Council,
}

#[cw_serde]
pub struct Proposal {
    pub proposal_type: ProposalType,
    pub id: ProposalId,
    pub proposer: Addr,
    pub title: String,
    pub description: String,
    pub status: ProposalStatus,
    pub started_at: Timestamp,
    pub expires: Expiration,
    pub proposal_actions: Vec<ProposalAction>,
    // TODO: include quorum? difficult because cw3 doesn't support it
    // pub quorum: Decimal,
}

// TODO: pagination for NFTs?
#[cw_serde]
pub struct UserStakeParams {
    pub user: String,
}

#[cw_serde]
pub struct UserStakeResponse {
    pub user_stake: UserStake,
}

#[cw_serde]
pub enum UserStake {
    Token(TokenUserStake),
    Nft(NftUserStake),
    None,
}

#[cw_serde]
pub struct TokenUserStake {
    pub amount: Uint128,
}

#[cw_serde]
pub struct NftUserStake {
    pub tokens: Vec<NftTokenId>,
    pub amount: Uint128,
}

#[cw_serde]
pub struct TotalStakedAmountResponse {
    pub total_staked_amount: Uint128,
}

#[cw_serde]
pub struct ClaimsResponse {
    pub claims: Vec<Claim>,
}

#[cw_serde]
pub struct ClaimsParams {
    pub owner: String,
}

/// Used as an alternative to CW721 spec's TokensResponse, because Talis doesn't actually
/// implement it correctly (they return 'ids' instead of 'tokens').
#[cw_serde]
pub struct TalisFriendlyTokensResponse {
    pub tokens: Option<Vec<String>>,
    pub ids: Option<Vec<String>>,
}

impl TalisFriendlyTokensResponse {
    pub fn to_tokens_response(self) -> StdResult<TokensResponse> {
        match self.tokens {
            None => match self.ids {
                None => Err(StdError::generic_err(
                    "Invalid CW721 TokensResponse, neither 'tokens' nor 'ids' field found",
                )),
                Some(ids) => Ok(TokensResponse { tokens: ids }),
            },
            Some(tokens) => Ok(TokensResponse { tokens }),
        }
    }
}
