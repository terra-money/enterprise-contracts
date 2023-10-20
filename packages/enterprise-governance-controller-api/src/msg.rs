use crate::api::{
    CastVoteMsg, ConfigResponse, CreateProposalMsg, CreateProposalWithNftDepositMsg,
    DaoCouncilSpec, ExecuteMsgReplyCallbackMsg, ExecuteProposalMsg, GovConfig, GovConfigResponse,
    MemberVoteParams, MemberVoteResponse, ProposalId, ProposalInfo, ProposalParams,
    ProposalResponse, ProposalStatusParams, ProposalStatusResponse, ProposalVotesParams,
    ProposalVotesResponse, ProposalsParams, ProposalsResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;
use enterprise_protocol::api::DaoType;
use membership_common_api::api::WeightsChangedMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_contract: String,
    pub dao_type: DaoType,
    pub gov_config: GovConfig,
    pub council_gov_config: Option<DaoCouncilSpec>,
    pub proposal_infos: Option<Vec<(ProposalId, ProposalInfo)>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateProposal(CreateProposalMsg),
    CreateProposalWithNftDeposit(CreateProposalWithNftDepositMsg),
    CreateCouncilProposal(CreateProposalMsg),
    CastVote(CastVoteMsg),
    CastCouncilVote(CastVoteMsg),
    ExecuteProposal(ExecuteProposalMsg),
    Receive(Cw20ReceiveMsg),
    WeightsChanged(WeightsChangedMsg),

    /// Callback from the ICS proxy contract.
    ExecuteMsgReplyCallback(ExecuteMsgReplyCallbackMsg),
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
    #[returns(ConfigResponse)]
    Config {},
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
