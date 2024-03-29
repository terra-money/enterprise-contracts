use crate::api::{
    CastVoteMsg, ConfigResponse, CreateProposalMsg, CreateProposalWithNftDepositMsg,
    DaoCouncilSpec, ExecuteProposalMsg, GovConfig, GovConfigResponse, MemberVoteParams,
    MemberVoteResponse, ProposalId, ProposalInfo, ProposalParams, ProposalResponse,
    ProposalStatusParams, ProposalStatusResponse, ProposalVotesParams, ProposalVotesResponse,
    ProposalsParams, ProposalsResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;
use enterprise_outposts_api::api::DeployCrossChainTreasuryMsg;
use enterprise_protocol::api::DaoType;
use membership_common_api::api::WeightsChangedMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_contract: String,
    pub dao_type: DaoType,
    pub gov_config: GovConfig,
    pub council_gov_config: Option<DaoCouncilSpec>,
    pub proposal_infos: Option<Vec<(ProposalId, ProposalInfo)>>,
    pub initial_cross_chain_treasuries: Option<Vec<DeployCrossChainTreasuryMsg>>,
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

    /// Only executable by the contract itself. Not part of the public API.
    ExecuteProposalActions(ExecuteProposalMsg),

    /// Only executable by the instantiator of this contract, in the same block as the creation.
    DeployInitialCrossChainTreasuries {},
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
