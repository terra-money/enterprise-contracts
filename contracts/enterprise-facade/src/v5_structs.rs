use common::commons::ModifyValue;
use common::commons::ModifyValue::{Change, NoChange};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Decimal, Timestamp, Uint128, Uint64};
use cw_asset::{AssetInfoUnchecked, AssetUnchecked};
use cw_utils::Duration;
use enterprise_facade_api::api::{
    AssetWhitelistParams, CastVoteMsg, ClaimsParams, CreateProposalMsg, DaoCouncil, DaoMetadata,
    DaoType, ExecuteProposalMsg, GovConfigV5, ListMultisigMembersMsg, Logo, MemberVoteParams,
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
pub struct UpdateMetadataV5Msg {
    pub name: ModifyValue<String>,
    pub description: ModifyValue<Option<String>>,
    pub logo: ModifyValue<Logo>,
    pub github_username: ModifyValue<Option<String>>,
    pub discord_username: ModifyValue<Option<String>>,
    pub twitter_username: ModifyValue<Option<String>>,
    pub telegram_username: ModifyValue<Option<String>>,
}

impl From<UpdateMetadataMsg> for UpdateMetadataV5Msg {
    fn from(value: UpdateMetadataMsg) -> Self {
        let logo = match value.logo {
            Change(logo) => Change(logo.into()),
            NoChange => NoChange,
        };

        UpdateMetadataV5Msg {
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
pub struct UpdateGovConfigV5Msg {
    pub quorum: ModifyValue<Decimal>,
    pub threshold: ModifyValue<Decimal>,
    pub veto_threshold: ModifyValue<Option<Decimal>>,
    pub voting_duration: ModifyValue<Uint64>,
    pub unlocking_period: ModifyValue<Duration>,
    pub minimum_deposit: ModifyValue<Option<Uint128>>,
    pub allow_early_proposal_execution: ModifyValue<bool>,
}

impl From<UpdateGovConfigMsg> for UpdateGovConfigV5Msg {
    fn from(value: UpdateGovConfigMsg) -> Self {
        UpdateGovConfigV5Msg {
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
pub struct UpdateCouncilV5Msg {
    pub dao_council: Option<DaoCouncilSpec>,
}

impl From<UpdateCouncilMsg> for UpdateCouncilV5Msg {
    fn from(value: UpdateCouncilMsg) -> Self {
        UpdateCouncilV5Msg {
            dao_council: value.dao_council,
        }
    }
}

#[cw_serde]
pub struct UpdateAssetWhitelistV5Msg {
    pub add: Vec<AssetInfoUnchecked>,
    pub remove: Vec<AssetInfoUnchecked>,
}

impl From<UpdateAssetWhitelistProposalActionMsg> for UpdateAssetWhitelistV5Msg {
    fn from(value: UpdateAssetWhitelistProposalActionMsg) -> Self {
        UpdateAssetWhitelistV5Msg {
            add: value.add,
            remove: value.remove,
        }
    }
}

#[cw_serde]
pub struct UpdateNftWhitelistV5Msg {
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

impl From<UpdateNftWhitelistProposalActionMsg> for UpdateNftWhitelistV5Msg {
    fn from(value: UpdateNftWhitelistProposalActionMsg) -> Self {
        UpdateNftWhitelistV5Msg {
            add: value.add,
            remove: value.remove,
        }
    }
}

#[cw_serde]
pub struct RequestFundingFromDaoV5Msg {
    pub recipient: String,
    pub assets: Vec<AssetUnchecked>,
}

impl From<RequestFundingFromDaoMsg> for RequestFundingFromDaoV5Msg {
    fn from(value: RequestFundingFromDaoMsg) -> Self {
        RequestFundingFromDaoV5Msg {
            recipient: value.recipient,
            assets: value.assets,
        }
    }
}

#[cw_serde]
pub struct UpgradeDaoV5Msg {
    pub new_dao_code_id: u64,
    pub migrate_msg: Binary,
}

#[cw_serde]
pub struct ExecuteMsgsV5Msg {
    pub action_type: String,
    pub msgs: Vec<String>,
}

impl From<ExecuteMsgsMsg> for ExecuteMsgsV5Msg {
    fn from(value: ExecuteMsgsMsg) -> Self {
        ExecuteMsgsV5Msg {
            action_type: value.action_type,
            msgs: value.msgs,
        }
    }
}

#[cw_serde]
pub struct ModifyMultisigMembershipV5Msg {
    /// Members to be edited.
    /// Can contain existing members, in which case their new weight will be the one specified in
    /// this message. This effectively allows removing of members (by setting their weight to 0).
    pub edit_members: Vec<MultisigMemberV5>,
}

impl From<ModifyMultisigMembershipMsg> for ModifyMultisigMembershipV5Msg {
    fn from(value: ModifyMultisigMembershipMsg) -> Self {
        ModifyMultisigMembershipV5Msg {
            edit_members: value.edit_members.into_iter().map(|it| it.into()).collect(),
        }
    }
}

#[cw_serde]
pub struct MultisigMemberV5 {
    pub address: String,
    pub weight: Uint128,
}

impl From<UserWeight> for MultisigMemberV5 {
    fn from(value: UserWeight) -> Self {
        MultisigMemberV5 {
            address: value.user,
            weight: value.weight,
        }
    }
}

#[cw_serde]
pub struct DistributeFundsV5Msg {
    pub funds: Vec<AssetUnchecked>,
}

impl From<DistributeFundsMsg> for DistributeFundsV5Msg {
    fn from(value: DistributeFundsMsg) -> Self {
        DistributeFundsV5Msg { funds: value.funds }
    }
}

#[cw_serde]
pub struct UpdateMinimumWeightForRewardsV5Msg {
    pub minimum_weight_for_rewards: Uint128,
}

impl From<UpdateMinimumWeightForRewardsMsg> for UpdateMinimumWeightForRewardsV5Msg {
    fn from(value: UpdateMinimumWeightForRewardsMsg) -> Self {
        UpdateMinimumWeightForRewardsV5Msg {
            minimum_weight_for_rewards: value.minimum_weight_for_rewards,
        }
    }
}

/// This is what execute messages for Enterprise contract looked like for v5.
#[cw_serde]
pub enum ExecuteV5Msg {
    // facaded part
    ExecuteProposal(ExecuteProposalMsg),
    // adapted part
    CreateProposal(CreateProposalV5Msg),
    CreateCouncilProposal(CreateProposalV5Msg),
    CastVote(CastVoteMsg),
    CastCouncilVote(CastVoteMsg),
    Unstake(UnstakeMsg),
    Claim {},
}

#[cw_serde]
pub struct CreateProposalV5Msg {
    /// Title of the proposal
    pub title: String,
    /// Optional description text of the proposal
    pub description: Option<String>,
    /// Actions to be executed, in order, if the proposal passes
    pub proposal_actions: Vec<ProposalActionV5>,
}

#[cw_serde]
pub enum ProposalActionV5 {
    UpdateMetadata(UpdateMetadataV5Msg),
    UpdateGovConfig(UpdateGovConfigV5Msg),
    UpdateCouncil(UpdateCouncilV5Msg),
    UpdateAssetWhitelist(UpdateAssetWhitelistV5Msg),
    UpdateNftWhitelist(UpdateNftWhitelistV5Msg),
    RequestFundingFromDao(RequestFundingFromDaoV5Msg),
    UpgradeDao(UpgradeDaoV5Msg),
    ExecuteMsgs(ExecuteMsgsV5Msg),
    ModifyMultisigMembership(ModifyMultisigMembershipV5Msg),
    DistributeFunds(DistributeFundsV5Msg),
    UpdateMinimumWeightForRewards(UpdateMinimumWeightForRewardsV5Msg),
}

/// This is what CW20-receive hook messages for Enterprise contract looked like for v5.
#[cw_serde]
pub enum Cw20HookV5Msg {
    Stake {},
    CreateProposal(CreateProposalMsg),
}

/// This is what CW721-receive hook messages for Enterprise contract looked like for v5.
#[cw_serde]
pub enum Cw721HookV5Msg {
    Stake {},
}

/// This is what query messages for Enterprise contract looked like for v5.
/// Looks almost the same as the API for enterprise-facade, but the facade also takes target
/// Enterprise contract address in each of the queries.
#[cw_serde]
pub enum QueryV5Msg {
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
    UserStake(UserStakeV5Params),
    TotalStakedAmount {},
    StakedNfts(StakedNftsParams),
    Claims(ClaimsParams),
    ReleasableClaims(ClaimsParams),
}

#[cw_serde]
pub struct UserStakeV5Params {
    pub user: String,
}

#[cw_serde]
pub struct DaoInfoResponseV5 {
    pub creation_date: Timestamp,
    pub metadata: DaoMetadata,
    pub gov_config: GovConfigV5,
    pub dao_council: Option<DaoCouncil>,
    pub dao_type: DaoType,
    pub dao_membership_contract: Addr,
    pub enterprise_factory_contract: Addr,
    pub funds_distributor_contract: Addr,
    pub dao_code_version: Uint64,
}
