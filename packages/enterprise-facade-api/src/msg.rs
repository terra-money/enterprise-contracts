use crate::api::{
    AdapterResponse, AssetWhitelistParams, AssetWhitelistResponse, CastVoteMsg, ClaimsParams,
    ClaimsResponse, CreateProposalMsg, CreateProposalWithDenomDepositMsg,
    CreateProposalWithTokenDepositMsg, DaoInfoResponse, ExecuteProposalMsg, ListMultisigMembersMsg,
    MemberInfoResponse, MemberVoteParams, MemberVoteResponse, MultisigMembersResponse,
    NftWhitelistParams, NftWhitelistResponse, ProposalParams, ProposalResponse,
    ProposalStatusParams, ProposalStatusResponse, ProposalVotesParams, ProposalVotesResponse,
    ProposalsParams, ProposalsResponse, QueryMemberInfoMsg, StakeMsg, StakedNftsParams,
    StakedNftsResponse, TotalStakedAmountResponse, UnstakeMsg, UserStakeParams, UserStakeResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use enterprise_governance_controller_api::api::CreateProposalWithNftDepositMsg;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    ExecuteProposal {
        contract: Addr,
        msg: ExecuteProposalMsg,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DaoInfoResponse)]
    DaoInfo { contract: Addr },
    #[returns(MemberInfoResponse)]
    MemberInfo {
        contract: Addr,
        msg: QueryMemberInfoMsg,
    },
    #[returns(MultisigMembersResponse)]
    ListMultisigMembers {
        contract: Addr,
        msg: ListMultisigMembersMsg,
    },
    #[returns(AssetWhitelistResponse)]
    AssetWhitelist {
        contract: Addr,
        params: AssetWhitelistParams,
    },
    #[returns(NftWhitelistResponse)]
    NftWhitelist {
        contract: Addr,
        params: NftWhitelistParams,
    },
    #[returns(ProposalResponse)]
    Proposal {
        contract: Addr,
        params: ProposalParams,
    },
    #[returns(ProposalsResponse)]
    Proposals {
        contract: Addr,
        params: ProposalsParams,
    },
    #[returns(ProposalStatusResponse)]
    ProposalStatus {
        contract: Addr,
        params: ProposalStatusParams,
    },
    #[returns(MemberVoteResponse)]
    MemberVote {
        contract: Addr,
        params: MemberVoteParams,
    },
    #[returns(ProposalVotesResponse)]
    ProposalVotes {
        contract: Addr,
        params: ProposalVotesParams,
    },
    #[returns(UserStakeResponse)]
    UserStake {
        contract: Addr,
        params: UserStakeParams,
    },
    #[returns(TotalStakedAmountResponse)]
    TotalStakedAmount { contract: Addr },
    #[returns(StakedNftsResponse)]
    StakedNfts {
        contract: Addr,
        params: StakedNftsParams,
    },
    #[returns(ClaimsResponse)]
    Claims {
        contract: Addr,
        params: ClaimsParams,
    },
    #[returns(ClaimsResponse)]
    ReleasableClaims {
        contract: Addr,
        params: ClaimsParams,
    },

    // Adapter queries - those are designed to be called to determine which contract should be
    // called with which message to achieve the desired result
    #[returns(AdapterResponse)]
    CreateProposalAdapted {
        contract: Addr,
        params: CreateProposalMsg,
    },

    #[returns(AdapterResponse)]
    CreateProposalWithDenomDepositAdapted {
        contract: Addr,
        params: CreateProposalWithDenomDepositMsg,
    },

    #[returns(AdapterResponse)]
    CreateProposalWithTokenDepositAdapted {
        contract: Addr,
        params: CreateProposalWithTokenDepositMsg,
    },

    #[returns(AdapterResponse)]
    CreateProposalWithNftDepositAdapted {
        contract: Addr,
        params: CreateProposalWithNftDepositMsg,
    },

    #[returns(AdapterResponse)]
    CreateCouncilProposalAdapted {
        contract: Addr,
        params: CreateProposalMsg,
    },

    #[returns(AdapterResponse)]
    CastVoteAdapted { contract: Addr, params: CastVoteMsg },

    #[returns(AdapterResponse)]
    CastCouncilVoteAdapted { contract: Addr, params: CastVoteMsg },

    #[returns(AdapterResponse)]
    StakeAdapted { contract: Addr, params: StakeMsg },

    #[returns(AdapterResponse)]
    UnstakeAdapted { contract: Addr, params: UnstakeMsg },

    #[returns(AdapterResponse)]
    ClaimAdapted { contract: Addr },
}
