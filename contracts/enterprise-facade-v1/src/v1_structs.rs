use common::commons::ModifyValue;
use common::commons::ModifyValue::{Change, NoChange};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Decimal, Timestamp, Uint128, Uint64};
use cw_asset::{AssetInfoUnchecked, AssetUnchecked};
use cw_utils::{Duration, Expiration};
use enterprise_facade_api::api::{
    AssetWhitelistParams, CastVoteMsg, ClaimsParams, DaoCouncil, DaoMetadata, DaoType,
    ExecuteProposalMsg, GovConfigV1, ListMultisigMembersMsg, Logo, MemberVoteParams, NftTokenId,
    NftWhitelistParams, Proposal, ProposalId, ProposalParams, ProposalResponse, ProposalStatus,
    ProposalStatusParams, ProposalType, ProposalVotesParams, ProposalsParams, ProposalsResponse,
    QueryMemberInfoMsg, StakedNftsParams,
};
use enterprise_governance_controller_api::api::{
    DaoCouncilSpec, DistributeFundsMsg, ExecuteMsgsMsg, ModifyMultisigMembershipMsg,
    ProposalAction, RequestFundingFromDaoMsg, UpdateAssetWhitelistProposalActionMsg,
    UpdateCouncilMsg, UpdateGovConfigMsg, UpdateMinimumWeightForRewardsMsg,
    UpdateNftWhitelistProposalActionMsg,
};
use enterprise_protocol::api::{UpdateMetadataMsg, UpgradeDaoMsg, VersionMigrateMsg};
use enterprise_versioning_api::api::Version;
use multisig_membership_api::api::UserWeight;
use serde_with::serde_as;
use std::collections::BTreeMap;

#[cw_serde]
pub struct UpdateMetadataV1Msg {
    pub name: ModifyValue<String>,
    pub description: ModifyValue<Option<String>>,
    pub logo: ModifyValue<Logo>,
    pub github_username: ModifyValue<Option<String>>,
    pub discord_username: ModifyValue<Option<String>>,
    pub twitter_username: ModifyValue<Option<String>>,
    pub telegram_username: ModifyValue<Option<String>>,
}

impl From<UpdateMetadataMsg> for UpdateMetadataV1Msg {
    fn from(value: UpdateMetadataMsg) -> Self {
        let logo = match value.logo {
            Change(logo) => Change(logo.into()),
            NoChange => NoChange,
        };

        UpdateMetadataV1Msg {
            name: value.name,
            description: value.description,
            logo,
            github_username: value.github_username,
            discord_username: value.discord_username,
            twitter_username: value.twitter_username,
            telegram_username: value.telegram_username,
        }
    }
}

impl From<UpdateMetadataV1Msg> for UpdateMetadataMsg {
    fn from(value: UpdateMetadataV1Msg) -> Self {
        let logo = match value.logo {
            Change(logo) => Change(logo.into()),
            NoChange => NoChange,
        };

        UpdateMetadataMsg {
            name: value.name,
            description: value.description,
            logo,
            github_username: value.github_username,
            discord_username: value.discord_username,
            twitter_username: value.twitter_username,
            telegram_username: value.telegram_username,
        }
    }
}

#[cw_serde]
pub struct UpdateGovConfigV1Msg {
    pub quorum: ModifyValue<Decimal>,
    pub threshold: ModifyValue<Decimal>,
    pub veto_threshold: ModifyValue<Option<Decimal>>,
    pub voting_duration: ModifyValue<Uint64>,
    pub unlocking_period: ModifyValue<Duration>,
    pub minimum_deposit: ModifyValue<Option<Uint128>>,
    pub allow_early_proposal_execution: ModifyValue<bool>,
}

impl From<UpdateGovConfigMsg> for UpdateGovConfigV1Msg {
    fn from(value: UpdateGovConfigMsg) -> Self {
        UpdateGovConfigV1Msg {
            quorum: value.quorum,
            threshold: value.threshold,
            veto_threshold: value.veto_threshold,
            voting_duration: value.voting_duration,
            unlocking_period: value.unlocking_period,
            minimum_deposit: value.minimum_deposit,
            allow_early_proposal_execution: value.allow_early_proposal_execution,
        }
    }
}

impl From<UpdateGovConfigV1Msg> for UpdateGovConfigMsg {
    fn from(value: UpdateGovConfigV1Msg) -> Self {
        UpdateGovConfigMsg {
            quorum: value.quorum,
            threshold: value.threshold,
            veto_threshold: value.veto_threshold,
            voting_duration: value.voting_duration,
            unlocking_period: value.unlocking_period,
            minimum_deposit: value.minimum_deposit,
            allow_early_proposal_execution: value.allow_early_proposal_execution,
        }
    }
}

#[cw_serde]
pub struct UpdateCouncilV1Msg {
    pub dao_council: Option<DaoCouncilSpec>,
}

impl From<UpdateCouncilMsg> for UpdateCouncilV1Msg {
    fn from(value: UpdateCouncilMsg) -> Self {
        UpdateCouncilV1Msg {
            dao_council: value.dao_council,
        }
    }
}

impl From<UpdateCouncilV1Msg> for UpdateCouncilMsg {
    fn from(value: UpdateCouncilV1Msg) -> Self {
        UpdateCouncilMsg {
            dao_council: value.dao_council,
        }
    }
}

#[cw_serde]
pub struct UpdateAssetWhitelistV1Msg {
    pub add: Vec<AssetInfoUnchecked>,
    pub remove: Vec<AssetInfoUnchecked>,
}

impl From<UpdateAssetWhitelistProposalActionMsg> for UpdateAssetWhitelistV1Msg {
    fn from(value: UpdateAssetWhitelistProposalActionMsg) -> Self {
        UpdateAssetWhitelistV1Msg {
            add: value.add,
            remove: value.remove,
        }
    }
}

impl From<UpdateAssetWhitelistV1Msg> for UpdateAssetWhitelistProposalActionMsg {
    fn from(value: UpdateAssetWhitelistV1Msg) -> Self {
        UpdateAssetWhitelistProposalActionMsg {
            remote_treasury_target: None,
            add: value.add,
            remove: value.remove,
        }
    }
}

#[cw_serde]
pub struct UpdateNftWhitelistV1Msg {
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

impl From<UpdateNftWhitelistProposalActionMsg> for UpdateNftWhitelistV1Msg {
    fn from(value: UpdateNftWhitelistProposalActionMsg) -> Self {
        UpdateNftWhitelistV1Msg {
            add: value.add,
            remove: value.remove,
        }
    }
}

impl From<UpdateNftWhitelistV1Msg> for UpdateNftWhitelistProposalActionMsg {
    fn from(value: UpdateNftWhitelistV1Msg) -> Self {
        UpdateNftWhitelistProposalActionMsg {
            remote_treasury_target: None,
            add: value.add,
            remove: value.remove,
        }
    }
}

#[cw_serde]
pub struct RequestFundingFromDaoV1Msg {
    pub recipient: String,
    pub assets: Vec<AssetUnchecked>,
}

impl From<RequestFundingFromDaoMsg> for RequestFundingFromDaoV1Msg {
    fn from(value: RequestFundingFromDaoMsg) -> Self {
        RequestFundingFromDaoV1Msg {
            recipient: value.recipient,
            assets: value.assets,
        }
    }
}

impl From<RequestFundingFromDaoV1Msg> for RequestFundingFromDaoMsg {
    fn from(value: RequestFundingFromDaoV1Msg) -> Self {
        RequestFundingFromDaoMsg {
            remote_treasury_target: None,
            recipient: value.recipient,
            assets: value.assets,
        }
    }
}

#[cw_serde]
pub struct UpgradeDaoV1Msg {
    pub new_dao_code_id: u64,
    pub migrate_msg: Binary,
}

impl From<UpgradeDaoV1Msg> for UpgradeDaoMsg {
    fn from(value: UpgradeDaoV1Msg) -> Self {
        UpgradeDaoMsg {
            new_version: Version {
                major: 0,
                minor: value.new_dao_code_id,
                patch: 0,
            },
            migrate_msgs: vec![VersionMigrateMsg {
                version: Version {
                    major: 0,
                    minor: value.new_dao_code_id,
                    patch: 0,
                },
                migrate_msg: value.migrate_msg,
            }],
        }
    }
}

#[cw_serde]
pub struct ExecuteMsgsV1Msg {
    pub action_type: String,
    pub msgs: Vec<String>,
}

impl From<ExecuteMsgsMsg> for ExecuteMsgsV1Msg {
    fn from(value: ExecuteMsgsMsg) -> Self {
        ExecuteMsgsV1Msg {
            action_type: value.action_type,
            msgs: value.msgs,
        }
    }
}

impl From<ExecuteMsgsV1Msg> for ExecuteMsgsMsg {
    fn from(value: ExecuteMsgsV1Msg) -> Self {
        ExecuteMsgsMsg {
            action_type: value.action_type,
            msgs: value.msgs,
        }
    }
}

#[cw_serde]
pub struct ModifyMultisigMembershipV1Msg {
    /// Members to be edited.
    /// Can contain existing members, in which case their new weight will be the one specified in
    /// this message. This effectively allows removing of members (by setting their weight to 0).
    pub edit_members: Vec<MultisigMemberV1>,
}

impl From<ModifyMultisigMembershipMsg> for ModifyMultisigMembershipV1Msg {
    fn from(value: ModifyMultisigMembershipMsg) -> Self {
        ModifyMultisigMembershipV1Msg {
            edit_members: value.edit_members.into_iter().map(|it| it.into()).collect(),
        }
    }
}

impl From<ModifyMultisigMembershipV1Msg> for ModifyMultisigMembershipMsg {
    fn from(value: ModifyMultisigMembershipV1Msg) -> Self {
        ModifyMultisigMembershipMsg {
            edit_members: value.edit_members.into_iter().map(|it| it.into()).collect(),
        }
    }
}

#[cw_serde]
pub struct MultisigMemberV1 {
    pub address: String,
    pub weight: Uint128,
}

impl From<UserWeight> for MultisigMemberV1 {
    fn from(value: UserWeight) -> Self {
        MultisigMemberV1 {
            address: value.user,
            weight: value.weight,
        }
    }
}

impl From<MultisigMemberV1> for UserWeight {
    fn from(value: MultisigMemberV1) -> Self {
        UserWeight {
            user: value.address,
            weight: value.weight,
        }
    }
}

#[cw_serde]
pub struct DistributeFundsV1Msg {
    pub funds: Vec<AssetUnchecked>,
}

impl From<DistributeFundsMsg> for DistributeFundsV1Msg {
    fn from(value: DistributeFundsMsg) -> Self {
        DistributeFundsV1Msg { funds: value.funds }
    }
}

impl From<DistributeFundsV1Msg> for DistributeFundsMsg {
    fn from(value: DistributeFundsV1Msg) -> Self {
        DistributeFundsMsg { funds: value.funds }
    }
}

#[cw_serde]
pub struct UpdateMinimumWeightForRewardsV1Msg {
    pub minimum_weight_for_rewards: Uint128,
}

impl From<UpdateMinimumWeightForRewardsMsg> for UpdateMinimumWeightForRewardsV1Msg {
    fn from(value: UpdateMinimumWeightForRewardsMsg) -> Self {
        UpdateMinimumWeightForRewardsV1Msg {
            minimum_weight_for_rewards: value.minimum_weight_for_rewards,
        }
    }
}

impl From<UpdateMinimumWeightForRewardsV1Msg> for UpdateMinimumWeightForRewardsMsg {
    fn from(value: UpdateMinimumWeightForRewardsV1Msg) -> Self {
        UpdateMinimumWeightForRewardsMsg {
            minimum_weight_for_rewards: value.minimum_weight_for_rewards,
        }
    }
}

/// This is what execute messages for Enterprise contract looked like for v1.
#[cw_serde]
pub enum ExecuteV1Msg {
    CreateProposal(CreateProposalV1Msg),
    CreateCouncilProposal(CreateProposalV1Msg),
    CastVote(CastVoteMsg),
    CastCouncilVote(CastVoteMsg),
    ExecuteProposal(ExecuteProposalMsg),
    Unstake(UnstakeV1Msg),
    Claim {},
}

#[cw_serde]
pub enum UnstakeV1Msg {
    Cw20(UnstakeCw20V1Msg),
    Cw721(UnstakeCw721V1Msg),
}

#[cw_serde]
pub struct UnstakeCw20V1Msg {
    pub amount: Uint128,
}

#[cw_serde]
pub struct UnstakeCw721V1Msg {
    pub tokens: Vec<NftTokenId>,
}

#[cw_serde]
pub struct CreateProposalV1Msg {
    /// Title of the proposal
    pub title: String,
    /// Optional description text of the proposal
    pub description: Option<String>,
    /// Actions to be executed, in order, if the proposal passes
    pub proposal_actions: Vec<ProposalActionV1>,
}

#[cw_serde]
pub enum ProposalActionV1 {
    UpdateMetadata(UpdateMetadataV1Msg),
    UpdateGovConfig(UpdateGovConfigV1Msg),
    UpdateCouncil(UpdateCouncilV1Msg),
    UpdateAssetWhitelist(UpdateAssetWhitelistV1Msg),
    UpdateNftWhitelist(UpdateNftWhitelistV1Msg),
    RequestFundingFromDao(RequestFundingFromDaoV1Msg),
    UpgradeDao(UpgradeDaoV1Msg),
    ExecuteMsgs(ExecuteMsgsV1Msg),
    ModifyMultisigMembership(ModifyMultisigMembershipV1Msg),
    DistributeFunds(DistributeFundsV1Msg),
    UpdateMinimumWeightForRewards(UpdateMinimumWeightForRewardsV1Msg),
}

#[cw_serde]
pub struct TreasuryV1_0_0MigrationMsg {
    pub initial_submsgs_limit: Option<u32>,
}

impl From<ProposalActionV1> for ProposalAction {
    fn from(value: ProposalActionV1) -> Self {
        match value {
            ProposalActionV1::UpdateMetadata(msg) => ProposalAction::UpdateMetadata(msg.into()),
            ProposalActionV1::UpdateGovConfig(msg) => ProposalAction::UpdateGovConfig(msg.into()),
            ProposalActionV1::UpdateCouncil(msg) => ProposalAction::UpdateCouncil(msg.into()),
            ProposalActionV1::UpdateAssetWhitelist(msg) => {
                ProposalAction::UpdateAssetWhitelist(msg.into())
            }
            ProposalActionV1::UpdateNftWhitelist(msg) => {
                ProposalAction::UpdateNftWhitelist(msg.into())
            }
            ProposalActionV1::RequestFundingFromDao(msg) => {
                ProposalAction::RequestFundingFromDao(msg.into())
            }
            ProposalActionV1::UpgradeDao(msg) => ProposalAction::UpgradeDao(msg.into()),
            ProposalActionV1::ExecuteMsgs(msg) => ProposalAction::ExecuteMsgs(msg.into()),
            ProposalActionV1::ModifyMultisigMembership(msg) => {
                ProposalAction::ModifyMultisigMembership(msg.into())
            }
            ProposalActionV1::DistributeFunds(msg) => ProposalAction::DistributeFunds(msg.into()),
            ProposalActionV1::UpdateMinimumWeightForRewards(msg) => {
                ProposalAction::UpdateMinimumWeightForRewards(msg.into())
            }
        }
    }
}

#[cw_serde]
pub enum ProposalStatusV1 {
    InProgress,
    Passed,
    Rejected,
    Executed,
}

impl From<ProposalStatusV1> for ProposalStatus {
    fn from(value: ProposalStatusV1) -> Self {
        match value {
            ProposalStatusV1::InProgress => ProposalStatus::InProgress,
            ProposalStatusV1::Passed => ProposalStatus::Passed,
            ProposalStatusV1::Rejected => ProposalStatus::Rejected,
            ProposalStatusV1::Executed => ProposalStatus::Executed,
        }
    }
}

#[cw_serde]
pub struct ProposalV1 {
    pub proposal_type: ProposalType,
    pub id: ProposalId,
    pub proposer: Option<Addr>,
    pub title: String,
    pub description: String,
    pub status: ProposalStatusV1,
    pub started_at: Timestamp,
    pub expires: Expiration,
    pub proposal_actions: Vec<ProposalActionV1>,
}

impl From<ProposalV1> for Proposal {
    fn from(value: ProposalV1) -> Self {
        Proposal {
            proposal_type: value.proposal_type,
            id: value.id,
            proposer: value.proposer,
            title: value.title,
            description: value.description,
            status: value.status.into(),
            started_at: value.started_at,
            expires: value.expires,
            proposal_actions: value
                .proposal_actions
                .into_iter()
                .map(|action| action.into())
                .collect(),
        }
    }
}

/// This is what CW20-receive hook messages for Enterprise contract looked like for v1.
#[cw_serde]
pub enum Cw20HookV1Msg {
    Stake {},
    CreateProposal(CreateProposalV1Msg),
}

/// This is what CW721-receive hook messages for Enterprise contract looked like for v1.
#[cw_serde]
pub enum Cw721HookV1Msg {
    Stake {},
}

/// This is what query messages for Enterprise contract looked like for v1.
/// Looks almost the same as the API for enterprise-facade, but the facade also takes target
/// Enterprise contract address in each of the queries.
#[cw_serde]
pub enum QueryV1Msg {
    DaoInfo {},
    MemberInfo(QueryMemberInfoMsg),
    ListMultisigMembers(ListMultisigMembersMsg),
    AssetWhitelist(AssetWhitelistParams),
    NftWhitelist(NftWhitelistParams),
    Proposal(ProposalParams),
    Proposals(ProposalsParams),
    ProposalStatus(ProposalStatusParams),
    MemberVote(MemberVoteParams),
    ProposalVotes(ProposalVotesParams),
    UserStake(UserStakeV1Params),
    TotalStakedAmount {},
    StakedNfts(StakedNftsParams),
    Claims(ClaimsParams),
    ReleasableClaims(ClaimsParams),
}

#[serde_as]
#[cw_serde]
pub struct ProposalResponseV1 {
    pub proposal: ProposalV1,

    pub proposal_status: ProposalStatusV1,

    #[schemars(with = "Vec<(u8, Uint128)>")]
    #[serde_as(as = "Vec<(_, _)>")]
    /// Total vote-count (value) for each outcome (key).
    pub results: BTreeMap<u8, u128>,

    pub total_votes_available: Uint128,
}

impl From<ProposalResponseV1> for ProposalResponse {
    fn from(value: ProposalResponseV1) -> Self {
        ProposalResponse {
            proposal: value.proposal.into(),
            proposal_status: value.proposal_status.into(),
            results: value.results,
            total_votes_available: value.total_votes_available,
        }
    }
}

#[cw_serde]
pub struct ProposalsResponseV1 {
    pub proposals: Vec<ProposalResponseV1>,
}

impl From<ProposalsResponseV1> for ProposalsResponse {
    fn from(value: ProposalsResponseV1) -> Self {
        ProposalsResponse {
            proposals: value
                .proposals
                .into_iter()
                .map(|proposal| proposal.into())
                .collect(),
        }
    }
}

#[cw_serde]
pub struct UserStakeV1Params {
    pub user: String,
}

#[cw_serde]
pub struct DaoInfoResponseV1 {
    pub creation_date: Timestamp,
    pub metadata: DaoMetadata,
    pub gov_config: GovConfigV1,
    pub dao_council: Option<DaoCouncil>,
    pub dao_type: DaoType,
    pub dao_membership_contract: Addr,
    pub enterprise_factory_contract: Addr,
    pub funds_distributor_contract: Addr,
    pub dao_code_version: Uint64,
}

#[cw_serde]
pub enum FundsDistributorExecuteV1Msg {
    ClaimRewards(ClaimRewardsV1Msg),
}

#[cw_serde]
pub struct ClaimRewardsV1Msg {
    pub user: String,
    /// Native denominations to be claimed
    pub native_denoms: Vec<String>,
    /// CW20 asset rewards to be claimed, should be addresses of CW20 tokens
    pub cw20_assets: Vec<String>,
}
