use crate::api::{
    AssetTreasuryResponse, AssetWhitelistResponse, CastVoteMsg, ClaimsParams, ClaimsResponse,
    CreateProposalMsg, DaoCouncilSpec, DaoGovConfig, DaoInfoResponse, DaoMembershipInfo,
    DaoMetadata, ExecuteProposalMsg, ListMultisigMembersMsg, MemberInfoResponse, MemberVoteParams,
    MemberVoteResponse, MultisigMembersResponse, NftWhitelistResponse, ProposalParams,
    ProposalResponse, ProposalStatusParams, ProposalStatusResponse, ProposalVotesParams,
    ProposalVotesResponse, ProposalsParams, ProposalsResponse, QueryMemberInfoMsg, ReceiveNftMsg,
    TotalStakedAmountResponse, UnstakeMsg, UserStakeParams, UserStakeResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20ReceiveMsg;
use cw_asset::AssetInfo;

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_governance_code_id: u64,
    pub funds_distributor_code_id: u64,
    pub dao_metadata: DaoMetadata,
    pub dao_gov_config: DaoGovConfig,
    /// Optional council structure that can manage certain aspects of the DAO
    pub dao_council: Option<DaoCouncilSpec>,
    pub dao_membership_info: DaoMembershipInfo,
    /// Address of enterprise-factory contract that is creating this DAO
    pub enterprise_factory_contract: String,
    /// Assets that are allowed to show in DAO's treasury
    pub asset_whitelist: Option<Vec<AssetInfo>>,
    /// NFTs (CW721) that are allowed to show in DAO's treasury
    pub nft_whitelist: Option<Vec<Addr>>,
    /// Minimum weight that a user should have in order to qualify for rewards.
    /// E.g. a value of 3 here means that a user in token or NFT DAO needs at least 3 staked
    /// DAO assets, or a weight of 3 in multisig DAO, to be eligible for rewards.
    pub minimum_weight_for_rewards: Option<Uint128>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateProposal(CreateProposalMsg),
    CreateCouncilProposal(CreateProposalMsg),
    CastVote(CastVoteMsg),
    CastCouncilVote(CastVoteMsg),
    ExecuteProposal(ExecuteProposalMsg),
    Unstake(UnstakeMsg),
    Claim {},
    Receive(Cw20ReceiveMsg),
    ReceiveNft(ReceiveNftMsg),
}

#[cw_serde]
pub enum Cw20HookMsg {
    Stake {},
    CreateProposal(CreateProposalMsg),
}

#[cw_serde]
pub enum Cw721HookMsg {
    Stake {},
}

#[cw_serde]
pub struct MigrateMsg {
    pub funds_distributor_code_id: u64,
    pub minimum_eligible_weight: Option<Uint128>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DaoInfoResponse)]
    DaoInfo {},
    #[returns(MemberInfoResponse)]
    MemberInfo(QueryMemberInfoMsg),
    #[returns(MultisigMembersResponse)]
    ListMultisigMembers(ListMultisigMembersMsg),
    #[returns(AssetWhitelistResponse)]
    AssetWhitelist {},
    #[returns(NftWhitelistResponse)]
    NftWhitelist {},
    #[returns(ProposalResponse)]
    Proposal(ProposalParams),
    #[returns(ProposalsResponse)]
    Proposals(ProposalsParams),
    #[returns(ProposalStatusResponse)]
    ProposalStatus(ProposalStatusParams),
    #[returns(MemberVoteResponse)]
    MemberVote(MemberVoteParams),
    #[returns(ProposalVotesResponse)]
    ProposalVotes(ProposalVotesParams),
    #[returns(UserStakeResponse)]
    UserStake(UserStakeParams),
    #[returns(TotalStakedAmountResponse)]
    TotalStakedAmount {},
    #[returns(ClaimsResponse)]
    Claims(ClaimsParams),
    #[returns(ClaimsResponse)]
    ReleasableClaims(ClaimsParams),
    #[returns(AssetTreasuryResponse)]
    Cw20Treasury {},
}
