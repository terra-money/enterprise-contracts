use common::cw::ReleaseAt;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Timestamp, Uint128, Uint64};
use cw_asset::{AssetInfo, AssetInfoUnchecked};
use cw_utils::{Duration, Expiration};
use enterprise_governance_controller_api::api::{ProposalAction, ProposalActionType};
use enterprise_versioning_api::api::Version;
use poll_engine_api::api::{Vote, VoteOutcome};
use serde_with::serde_as;
use std::collections::BTreeMap;
use std::fmt;
use strum_macros::Display;

pub type ProposalId = u64;
pub type NftTokenId = String;

#[cw_serde]
#[derive(Display)]
pub enum DaoType {
    Denom,
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

impl From<enterprise_protocol::api::Logo> for Logo {
    fn from(value: enterprise_protocol::api::Logo) -> Self {
        match value {
            enterprise_protocol::api::Logo::Url(url) => Logo::Url(url),
            enterprise_protocol::api::Logo::None => Logo::None,
        }
    }
}

impl From<Logo> for enterprise_protocol::api::Logo {
    fn from(value: Logo) -> Self {
        match value {
            Logo::Url(url) => enterprise_protocol::api::Logo::Url(url),
            Logo::None => enterprise_protocol::api::Logo::None,
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
pub struct DaoCouncil {
    pub members: Vec<Addr>,
    pub allowed_proposal_action_types: Vec<ProposalActionType>,
    pub quorum: Decimal,
    pub threshold: Decimal,
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

#[cw_serde]
pub struct CreateProposalWithDenomDepositMsg {
    pub create_proposal_msg: CreateProposalMsg,
    pub deposit_amount: Uint128,
}

#[cw_serde]
pub struct CreateProposalWithTokenDepositMsg {
    pub create_proposal_msg: CreateProposalMsg,
    pub deposit_amount: Uint128,
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
pub enum StakeMsg {
    Cw20(StakeCw20Msg),
    Cw721(StakeCw721Msg),
    Denom(StakeDenomMsg),
}

#[cw_serde]
pub struct StakeCw20Msg {
    pub user: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct StakeCw721Msg {
    pub user: String,
    pub tokens: Vec<NftTokenId>,
}

#[cw_serde]
pub struct StakeDenomMsg {
    pub amount: Uint128,
}

#[cw_serde]
pub enum UnstakeMsg {
    Cw20(UnstakeCw20Msg),
    Cw721(UnstakeCw721Msg),
    Denom(UnstakeDenomMsg),
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
pub struct UnstakeDenomMsg {
    pub amount: Uint128,
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
    Denom(DenomClaimAsset),
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
pub struct DenomClaimAsset {
    pub amount: Uint128,
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
pub struct TreasuryAddressResponse {
    pub treasury_address: Addr,
}

#[cw_serde]
pub struct DaoInfoResponse {
    pub creation_date: Timestamp,
    pub metadata: DaoMetadata,
    pub gov_config: GovConfigV1,
    pub dao_council: Option<DaoCouncil>,
    pub dao_type: DaoType,
    pub dao_membership_contract: String,
    pub enterprise_factory_contract: Addr,
    pub funds_distributor_contract: Addr,
    pub dao_code_version: Uint64,
    pub dao_version: Version,
}

#[cw_serde]
pub struct GovConfigV1 {
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
pub struct AssetWhitelistParams {
    pub start_after: Option<AssetInfoUnchecked>,
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
    InProgressCanExecuteEarly,
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

#[cw_serde]
pub struct UserStakeParams {
    pub user: String,
    pub start_after: Option<NftTokenId>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct StakedNftsParams {
    pub start_after: Option<NftTokenId>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct UserStakeResponse {
    pub user_stake: UserStake,
}

#[cw_serde]
pub enum UserStake {
    Denom(DenomUserStake),
    Token(TokenUserStake),
    Nft(NftUserStake),
    None,
}

#[cw_serde]
pub struct DenomUserStake {
    pub amount: Uint128,
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
pub struct StakedNftsResponse {
    pub nfts: Vec<NftTokenId>,
}

#[cw_serde]
pub struct ClaimsResponse {
    pub claims: Vec<Claim>,
}

#[cw_serde]
pub struct ClaimsParams {
    pub owner: String,
}

/// Used to enable adapter-like behavior, where this contract can tell its consumers what call to
/// make with which pre-compiled message in order to achieve desired behavior, regardless of
/// Enterprise version being used.
#[cw_serde]
pub struct AdapterResponse {
    pub msgs: Vec<AdaptedMsg>,
}

pub fn adapter_response_single_execute_msg(
    target_contract: Addr,
    msg: String,
    funds: Vec<Coin>,
) -> AdapterResponse {
    AdapterResponse {
        msgs: vec![AdaptedMsg::Execute(AdaptedExecuteMsg {
            target_contract,
            msg,
            funds,
        })],
    }
}

/// Used to enable adapter-like behavior, where this contract can tell its consumers what call to
/// make with which pre-compiled message in order to achieve desired behavior, regardless of
/// Enterprise version being used.
#[cw_serde]
pub enum AdaptedMsg {
    Execute(AdaptedExecuteMsg),
    Bank(AdaptedBankMsg),
}

#[cw_serde]
pub struct AdaptedExecuteMsg {
    pub target_contract: Addr,
    pub msg: String,
    pub funds: Vec<Coin>,
}

#[cw_serde]
pub struct AdaptedBankMsg {
    pub receiver: Addr,
    pub funds: Vec<Coin>,
}
