use crate::api::{
    AdapterResponse, AssetWhitelistParams, AssetWhitelistResponse, CastVoteMsg, ClaimsParams,
    ClaimsResponse, ComponentContractsResponse, CreateProposalMsg,
    CreateProposalWithDenomDepositMsg, CreateProposalWithTokenDepositMsg, DaoInfoResponse,
    ExecuteProposalMsg, ListMultisigMembersMsg, MemberInfoResponse, MemberVoteParams,
    MemberVoteResponse, MultisigMembersResponse, NftWhitelistParams, NftWhitelistResponse,
    ProposalParams, ProposalResponse, ProposalStatusParams, ProposalStatusResponse,
    ProposalVotesParams, ProposalVotesResponse, ProposalsParams, ProposalsResponse,
    QueryMemberInfoMsg, StakeMsg, StakedNftsParams, StakedNftsResponse, TotalStakedAmountResponse,
    TreasuryAddressResponse, UnstakeMsg, UserStakeParams, UserStakeResponse,
    V2MigrationStageResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use enterprise_governance_controller_api::api::CreateProposalWithNftDepositMsg;
use enterprise_outposts_api::api::{CrossChainTreasuriesParams, CrossChainTreasuriesResponse};
use enterprise_treasury_api::api::{
    HasIncompleteV2MigrationResponse, HasUnmovedStakesOrClaimsResponse,
};
use membership_common_api::api::{MembersParams, MembersResponse};

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_facade_v1: String,
    pub enterprise_facade_v2: String,
}

#[cw_serde]
pub struct ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(TreasuryAddressResponse)]
    TreasuryAddress { contract: Addr },
    #[returns(DaoInfoResponse)]
    DaoInfo { contract: Addr },
    #[returns(ComponentContractsResponse)]
    ComponentContracts { contract: Addr },
    #[returns(MemberInfoResponse)]
    MemberInfo {
        contract: Addr,
        msg: QueryMemberInfoMsg,
    },
    #[returns(MembersResponse)]
    Members { contract: Addr, msg: MembersParams },
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

    #[returns(CrossChainTreasuriesResponse)]
    CrossChainTreasuries {
        contract: Addr,
        params: CrossChainTreasuriesParams,
    },

    #[returns(HasIncompleteV2MigrationResponse)]
    HasIncompleteV2Migration { contract: Addr },

    #[returns(HasUnmovedStakesOrClaimsResponse)]
    HasUnmovedStakesOrClaims { contract: Addr },

    #[returns(V2MigrationStageResponse)]
    V2MigrationStage { contract: Addr },

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
    ExecuteProposalAdapted {
        contract: Addr,
        params: ExecuteProposalMsg,
    },

    #[returns(AdapterResponse)]
    StakeAdapted { contract: Addr, params: StakeMsg },

    #[returns(AdapterResponse)]
    UnstakeAdapted { contract: Addr, params: UnstakeMsg },

    #[returns(AdapterResponse)]
    ClaimAdapted { contract: Addr },
}

#[cw_serde]
pub struct MigrateMsg {
    pub enterprise_facade_v1: Option<String>,
    pub enterprise_facade_v2: Option<String>,
}
