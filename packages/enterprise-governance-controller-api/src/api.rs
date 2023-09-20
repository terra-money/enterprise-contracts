use common::commons::ModifyValue;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, BlockInfo, Decimal, Event, Timestamp, Uint128, Uint64};
use cw_asset::{AssetInfoUnchecked, AssetUnchecked};
use cw_utils::{Duration, Expiration};
use enterprise_protocol::api::{UpdateMetadataMsg, UpgradeDaoMsg};
use multisig_membership_api::api::UserWeight;
use poll_engine_api::api::{Vote, VoteOutcome};
use serde_with::serde_as;
use std::collections::BTreeMap;
use strum_macros::Display;

pub type ProposalId = u64;

#[cw_serde]
pub struct ProposalInfo {
    pub proposal_type: ProposalType,
    pub executed_at: Option<BlockInfo>,
    pub proposal_deposit: Option<ProposalDeposit>,
    pub proposal_actions: Vec<ProposalAction>,
}

#[cw_serde]
pub struct GovConfig {
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
pub struct CouncilGovConfig {
    pub allowed_proposal_action_types: Vec<ProposalActionType>,
    pub quorum: Decimal,
    pub threshold: Decimal,
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
    pub asset: ProposalAsset,
}

#[cw_serde]
pub enum ProposalAsset {
    Denom { denom: String, amount: Uint128 },
    Cw20 { token_addr: Addr, amount: Uint128 },
}

impl ProposalDeposit {
    pub fn amount(&self) -> Uint128 {
        match self.asset {
            ProposalAsset::Denom { amount, .. } => amount,
            ProposalAsset::Cw20 { amount, .. } => amount,
        }
    }
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
    AddAttestation,
    RemoveAttestation,
    DeployCrossChainTreasury,
}

#[cw_serde]
pub enum ProposalAction {
    UpdateMetadata(UpdateMetadataMsg),
    UpdateGovConfig(UpdateGovConfigMsg),
    UpdateCouncil(UpdateCouncilMsg),
    UpdateAssetWhitelist(UpdateAssetWhitelistProposalActionMsg),
    UpdateNftWhitelist(UpdateNftWhitelistProposalActionMsg),
    RequestFundingFromDao(RequestFundingFromDaoMsg),
    UpgradeDao(UpgradeDaoMsg),
    ExecuteMsgs(ExecuteMsgsMsg),
    ModifyMultisigMembership(ModifyMultisigMembershipMsg),
    DistributeFunds(DistributeFundsMsg),
    UpdateMinimumWeightForRewards(UpdateMinimumWeightForRewardsMsg),
    AddAttestation(AddAttestationMsg),
    RemoveAttestation {},
    DeployCrossChainTreasury(DeployCrossChainTreasuryMsg),
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
pub struct RequestFundingFromDaoMsg {
    pub remote_treasury_target: Option<RemoteTreasuryTarget>,
    pub recipient: String,
    pub assets: Vec<AssetUnchecked>,
}

#[cw_serde]
pub struct UpdateAssetWhitelistProposalActionMsg {
    pub remote_treasury_target: Option<RemoteTreasuryTarget>,
    /// New assets to add to the whitelist. Will ignore assets that are already whitelisted.
    pub add: Vec<AssetInfoUnchecked>,
    /// Assets to remove from the whitelist. Will ignore assets that are not already whitelisted.
    pub remove: Vec<AssetInfoUnchecked>,
}

#[cw_serde]
pub struct UpdateNftWhitelistProposalActionMsg {
    pub remote_treasury_target: Option<RemoteTreasuryTarget>,
    /// New NFTs to add to the whitelist. Will ignore NFTs that are already whitelisted.
    pub add: Vec<String>,
    /// NFTs to remove from the whitelist. Will ignore NFTs that are not already whitelisted.
    pub remove: Vec<String>,
}

#[cw_serde]
pub struct RemoteTreasuryTarget {
    /// Spec for the cross-chain message to send.
    /// Treasury address will be determined using chain-id given in the spec.
    pub cross_chain_msg_spec: CrossChainMsgSpec,
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
    pub edit_members: Vec<UserWeight>,
}

#[cw_serde]
pub struct DistributeFundsMsg {
    pub funds: Vec<AssetUnchecked>,
}

#[cw_serde]
pub struct UpdateMinimumWeightForRewardsMsg {
    pub minimum_weight_for_rewards: Uint128,
}

#[cw_serde]
pub struct AddAttestationMsg {
    pub attestation_text: String,
}

#[cw_serde]
pub struct DeployCrossChainTreasuryMsg {
    pub cross_chain_msg_spec: CrossChainMsgSpec,
    pub asset_whitelist: Option<Vec<AssetInfoUnchecked>>,
    pub nft_whitelist: Option<Vec<String>>,
    pub ics_proxy_code_id: u64,
    pub enterprise_treasury_code_id: u64,
    /// Proxy contract serving globally for the given chain, with no specific permission model.
    pub chain_global_proxy: String,
}

#[cw_serde]
pub struct CrossChainMsgSpec {
    pub chain_id: String,
    pub src_ibc_port: String,
    pub src_ibc_channel: String,
    pub dest_ibc_port: String,
    pub dest_ibc_channel: String,
    /// uluna IBC denom on the remote chain. Currently, can be calculated as 'ibc/' + uppercase(sha256('{port}/{channel}/uluna'))
    pub uluna_denom: String,
    /// Optional timeout for the cross-chain messages. Formatted in nanoseconds.
    pub timeout_nanos: Option<u64>,
}

#[cw_serde]
pub struct ExecuteMsgReplyCallbackMsg {
    pub callback_id: u32,
    pub events: Vec<Event>,
    pub data: Option<Binary>,
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
pub struct ConfigResponse {
    pub enterprise_contract: Addr,
}

#[cw_serde]
pub struct GovConfigResponse {
    pub gov_config: GovConfig,
    pub council_gov_config: Option<CouncilGovConfig>,
    pub dao_membership_contract: Addr,
    pub dao_council_membership_contract: Addr,
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
