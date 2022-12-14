use crate::api::{
    AssetTreasuryResponse, AssetWhitelistResponse, CastVoteMsg, ClaimsParams, ClaimsResponse,
    CreateProposalMsg, DaoGovConfig, DaoInfoResponse, DaoMembershipInfo, DaoMetadata,
    ExecuteProposalMsg, ListMultisigMembersMsg, MemberInfoResponse, MemberVoteParams,
    MemberVoteResponse, MultisigMembersResponse, NftTreasuryResponse, NftWhitelistResponse,
    ProposalParams, ProposalResponse, ProposalStatusParams, ProposalStatusResponse,
    ProposalVotesParams, ProposalVotesResponse, ProposalsParams, ProposalsResponse,
    QueryMemberInfoMsg, TotalStakedAmountResponse, UnstakeMsg, UserStakeParams, UserStakeResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ReceiveMsg;
use cw_asset::AssetInfo;

#[cw_serde]
pub struct InstantiateMsg {
    pub dao_metadata: DaoMetadata,
    pub dao_gov_config: DaoGovConfig,
    pub dao_membership_info: DaoMembershipInfo,
    /// Address of enterprise-factory contract that is creating this DAO
    pub enterprise_factory_contract: String,
    /// Assets that are allowed to show in DAO's treasury
    pub asset_whitelist: Option<Vec<AssetInfo>>,
    /// NFTs (CW721) that are allowed to show in DAO's treasury
    pub nft_whitelist: Option<Vec<Addr>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateProposal(CreateProposalMsg),
    CastVote(CastVoteMsg),
    ExecuteProposal(ExecuteProposalMsg),
    Unstake(UnstakeMsg),
    Claim {},
    Receive(Cw20ReceiveMsg),
    ReceiveNft(Cw721ReceiveMsg),
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
pub struct MigrateMsg {}

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
    Cw20Treasury {}, // TODO: allow pagination? but how?
    #[returns(NftTreasuryResponse)]
    NftTreasury {}, // TODO: allow pagination? but how?
}
