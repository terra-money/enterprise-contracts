use common::commons::ModifyValue;
use common::commons::ModifyValue::{Change, NoChange};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Decimal, Timestamp, Uint128, Uint64};
use cw_asset::{AssetInfoUnchecked, AssetUnchecked};
use cw_utils::Duration;
use enterprise_facade_api::api::{
    AssetWhitelistParams, CastVoteMsg, ClaimsParams, CreateProposalMsg, DaoCouncil, DaoMetadata,
    DaoType, ExecuteProposalMsg, GovConfigV1, ListMultisigMembersMsg, Logo, MemberVoteParams,
    NftWhitelistParams, ProposalParams, ProposalStatusParams, ProposalVotesParams, ProposalsParams,
    QueryMemberInfoMsg, StakedNftsParams, UnstakeMsg,
};
use enterprise_governance_controller_api::api::{
    DaoCouncilSpec, DistributeFundsMsg, ExecuteMsgsMsg, ModifyMultisigMembershipMsg,
    RequestFundingFromDaoMsg, UpdateAssetWhitelistProposalActionMsg, UpdateCouncilMsg,
    UpdateGovConfigMsg, UpdateMinimumWeightForRewardsMsg, UpdateNftWhitelistProposalActionMsg,
};
use enterprise_protocol::api::UpdateMetadataMsg;
use multisig_membership_api::api::UserWeight;

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

#[cw_serde]
pub struct UpgradeDaoV1Msg {
    pub new_dao_code_id: u64,
    pub migrate_msg: Binary,
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

#[cw_serde]
pub struct DistributeFundsV1Msg {
    pub funds: Vec<AssetUnchecked>,
}

impl From<DistributeFundsMsg> for DistributeFundsV1Msg {
    fn from(value: DistributeFundsMsg) -> Self {
        DistributeFundsV1Msg { funds: value.funds }
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

/// This is what execute messages for Enterprise contract looked like for v1.
#[cw_serde]
pub enum ExecuteV1Msg {
    // facaded part
    ExecuteProposal(ExecuteProposalMsg),
    // adapted part
    CreateProposal(CreateProposalV1Msg),
    CreateCouncilProposal(CreateProposalV1Msg),
    CastVote(CastVoteMsg),
    CastCouncilVote(CastVoteMsg),
    Unstake(UnstakeMsg),
    Claim {},
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

/// This is what CW20-receive hook messages for Enterprise contract looked like for v1.
#[cw_serde]
pub enum Cw20HookV1Msg {
    Stake {},
    CreateProposal(CreateProposalMsg),
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
