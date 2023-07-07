use crate::api::{
    CastVoteMsg, CreateProposalMsg, DaoCouncilSpec, ExecuteProposalMsg, GovConfig,
    GovConfigResponse, MemberVoteParams, MemberVoteResponse, ProposalId, ProposalInfo,
    ProposalParams, ProposalResponse, ProposalStatusParams, ProposalStatusResponse,
    ProposalVotesParams, ProposalVotesResponse, ProposalsParams, ProposalsResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;
use membership_common_api::api::WeightsChangedMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_contract: String,
    pub gov_config: GovConfig,
    pub council_gov_config: Option<DaoCouncilSpec>,
    pub proposal_infos: Option<Vec<(ProposalId, ProposalInfo)>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateProposal(CreateProposalMsg),
    CreateCouncilProposal(CreateProposalMsg),
    CastVote(CastVoteMsg),
    CastCouncilVote(CastVoteMsg),
    ExecuteProposal(ExecuteProposalMsg),
    Receive(Cw20ReceiveMsg),
    WeightsChanged(WeightsChangedMsg),

    /// Only executable by the contract itself. Not part of the public API.
    ExecuteProposalActions(ExecuteProposalMsg),
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
