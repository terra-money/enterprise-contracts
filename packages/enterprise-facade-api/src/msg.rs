use crate::api::{
    AssetWhitelistParams, AssetWhitelistResponse, CastVoteMsg, ClaimsParams, ClaimsResponse,
    CreateProposalMsg, DaoInfoResponse, ExecuteProposalMsg, ListMultisigMembersMsg,
    MemberInfoResponse, MemberVoteParams, MemberVoteResponse, MultisigMembersResponse,
    NftWhitelistParams, NftWhitelistResponse, ProposalParams, ProposalResponse,
    ProposalStatusParams, ProposalStatusResponse, ProposalVotesParams, ProposalVotesResponse,
    ProposalsParams, ProposalsResponse, QueryMemberInfoMsg, ReceiveNftMsg, StakedNftsParams,
    StakedNftsResponse, TotalStakedAmountResponse, UnstakeMsg, UserStakeParams, UserStakeResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    ExecuteProposal(ExecuteProposalMsg),
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
    AssetWhitelist(AssetWhitelistParams),
    #[returns(NftWhitelistResponse)]
    NftWhitelist(NftWhitelistParams),
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
    #[returns(StakedNftsResponse)]
    StakedNfts(StakedNftsParams),
    #[returns(ClaimsResponse)]
    Claims(ClaimsParams),
    #[returns(ClaimsResponse)]
    ReleasableClaims(ClaimsParams),
}
