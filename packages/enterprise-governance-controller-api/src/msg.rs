use crate::api::{
    CastVoteMsg, CreateProposalMsg, ExecuteProposalMsg, GovConfig, GovConfigResponse,
    MemberVoteParams, MemberVoteResponse, ProposalParams, ProposalResponse, ProposalStatusParams,
    ProposalStatusResponse, ProposalVotesParams, ProposalVotesResponse, ProposalsParams,
    ProposalsResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_contract: String,
    pub dao_council_membership_contract: String,
    pub gov_config: GovConfig,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateProposal(CreateProposalMsg),
    CreateCouncilProposal(CreateProposalMsg),
    CastVote(CastVoteMsg),
    CastCouncilVote(CastVoteMsg),
    ExecuteProposal(ExecuteProposalMsg),
    ExecuteProposalActions(ExecuteProposalMsg),
    Receive(Cw20ReceiveMsg),
}

#[cw_serde]
pub enum Cw20HookMsg {
    CreateProposal(CreateProposalMsg),
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GovConfigResponse)]
    GovConfig {},
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
}
