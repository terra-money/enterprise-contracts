use crate::facade::EnterpriseFacade;
use crate::facade_v5::ExecuteV5Msg::{
    CastCouncilVote, CastVote, Claim, CreateCouncilProposal, CreateProposal, Unstake,
};
use crate::facade_v5::QueryV5Msg::{
    AssetWhitelist, Claims, DaoInfo, ListMultisigMembers, MemberInfo, MemberVote, NftWhitelist,
    Proposal, ProposalVotes, Proposals, ReleasableClaims, StakedNfts, TotalStakedAmount, UserStake,
};
use common::cw::{Context, QueryContext};
use cosmwasm_schema::cw_serde;
use cosmwasm_schema::serde::de::DeserializeOwned;
use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{wasm_execute, Addr, Deps, Response, SubMsg};
use enterprise_facade_api::api::{
    AdapterResponse, AssetWhitelistParams, AssetWhitelistResponse, CastVoteMsg, ClaimsParams,
    ClaimsResponse, CreateProposalMsg, DaoInfoResponse, ExecuteProposalMsg, ListMultisigMembersMsg,
    MemberInfoResponse, MemberVoteParams, MemberVoteResponse, MultisigMembersResponse,
    NftWhitelistParams, NftWhitelistResponse, ProposalParams, ProposalResponse,
    ProposalStatusParams, ProposalStatusResponse, ProposalVotesParams, ProposalVotesResponse,
    ProposalsParams, ProposalsResponse, QueryMemberInfoMsg, StakedNftsParams, StakedNftsResponse,
    TotalStakedAmountResponse, UnstakeMsg, UserStakeParams, UserStakeResponse,
};
use enterprise_facade_api::error::EnterpriseFacadeResult;
use QueryV5Msg::ProposalStatus;

/// Facade implementation for v0.5.0 of Enterprise (pre-contract-rewrite).
pub struct EnterpriseFacadeV5 {
    pub enterprise_address: Addr,
}

impl EnterpriseFacade for EnterpriseFacadeV5 {
    fn execute_proposal(
        &self,
        _ctx: &mut Context,
        msg: ExecuteProposalMsg,
    ) -> EnterpriseFacadeResult<Response> {
        let submsg = SubMsg::new(wasm_execute(
            self.enterprise_address.to_string(),
            &ExecuteV5Msg::ExecuteProposal(msg),
            vec![],
        )?);
        Ok(Response::new().add_submessage(submsg))
    }

    fn query_dao_info(&self, qctx: QueryContext) -> EnterpriseFacadeResult<DaoInfoResponse> {
        self.query_enterprise_contract(qctx.deps, &DaoInfo {})
    }

    fn query_member_info(
        &self,
        qctx: QueryContext,
        msg: QueryMemberInfoMsg,
    ) -> EnterpriseFacadeResult<MemberInfoResponse> {
        self.query_enterprise_contract(qctx.deps, &MemberInfo(msg))
    }

    fn query_list_multisig_members(
        &self,
        qctx: QueryContext,
        msg: ListMultisigMembersMsg,
    ) -> EnterpriseFacadeResult<MultisigMembersResponse> {
        self.query_enterprise_contract(qctx.deps, &ListMultisigMembers(msg))
    }

    fn query_asset_whitelist(
        &self,
        qctx: QueryContext,
        params: AssetWhitelistParams,
    ) -> EnterpriseFacadeResult<AssetWhitelistResponse> {
        self.query_enterprise_contract(qctx.deps, &AssetWhitelist(params))
    }

    fn query_nft_whitelist(
        &self,
        qctx: QueryContext,
        params: NftWhitelistParams,
    ) -> EnterpriseFacadeResult<NftWhitelistResponse> {
        self.query_enterprise_contract(qctx.deps, &NftWhitelist(params))
    }

    fn query_proposal(
        &self,
        qctx: QueryContext,
        params: ProposalParams,
    ) -> EnterpriseFacadeResult<ProposalResponse> {
        self.query_enterprise_contract(qctx.deps, &Proposal(params))
    }

    fn query_proposals(
        &self,
        qctx: QueryContext,
        params: ProposalsParams,
    ) -> EnterpriseFacadeResult<ProposalsResponse> {
        self.query_enterprise_contract(qctx.deps, &Proposals(params))
    }

    fn query_proposal_status(
        &self,
        qctx: QueryContext,
        params: ProposalStatusParams,
    ) -> EnterpriseFacadeResult<ProposalStatusResponse> {
        self.query_enterprise_contract(qctx.deps, &ProposalStatus(params))
    }

    fn query_member_vote(
        &self,
        qctx: QueryContext,
        params: MemberVoteParams,
    ) -> EnterpriseFacadeResult<MemberVoteResponse> {
        self.query_enterprise_contract(qctx.deps, &MemberVote(params))
    }

    fn query_proposal_votes(
        &self,
        qctx: QueryContext,
        params: ProposalVotesParams,
    ) -> EnterpriseFacadeResult<ProposalVotesResponse> {
        self.query_enterprise_contract(qctx.deps, &ProposalVotes(params))
    }

    fn query_user_stake(
        &self,
        qctx: QueryContext,
        params: UserStakeParams,
    ) -> EnterpriseFacadeResult<UserStakeResponse> {
        self.query_enterprise_contract(
            qctx.deps,
            &UserStake(UserStakeV5Params { user: params.user }),
        )
    }

    fn query_total_staked_amount(
        &self,
        qctx: QueryContext,
    ) -> EnterpriseFacadeResult<TotalStakedAmountResponse> {
        self.query_enterprise_contract(qctx.deps, &TotalStakedAmount {})
    }

    fn query_staked_nfts(
        &self,
        qctx: QueryContext,
        params: StakedNftsParams,
    ) -> EnterpriseFacadeResult<StakedNftsResponse> {
        self.query_enterprise_contract(qctx.deps, &StakedNfts(params))
    }

    fn query_claims(
        &self,
        qctx: QueryContext,
        params: ClaimsParams,
    ) -> EnterpriseFacadeResult<ClaimsResponse> {
        self.query_enterprise_contract(qctx.deps, &Claims(params))
    }

    fn query_releasable_claims(
        &self,
        qctx: QueryContext,
        params: ClaimsParams,
    ) -> EnterpriseFacadeResult<ClaimsResponse> {
        self.query_enterprise_contract(qctx.deps, &ReleasableClaims(params))
    }

    fn adapt_create_proposal(
        &self,
        _: QueryContext,
        params: CreateProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(AdapterResponse {
            target_contract: self.enterprise_address.clone(),
            msg: serde_json::to_string(&CreateProposal(params))?,
        })
    }

    fn adapt_create_council_proposal(
        &self,
        _: QueryContext,
        params: CreateProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(AdapterResponse {
            target_contract: self.enterprise_address.clone(),
            msg: serde_json::to_string(&CreateCouncilProposal(params))?,
        })
    }

    fn adapt_cast_vote(
        &self,
        _: QueryContext,
        params: CastVoteMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(AdapterResponse {
            target_contract: self.enterprise_address.clone(),
            msg: serde_json::to_string(&CastVote(params))?,
        })
    }

    fn adapt_cast_council_vote(
        &self,
        _: QueryContext,
        params: CastVoteMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(AdapterResponse {
            target_contract: self.enterprise_address.clone(),
            msg: serde_json::to_string(&CastCouncilVote(params))?,
        })
    }

    fn adapt_unstake(
        &self,
        _: QueryContext,
        params: UnstakeMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(AdapterResponse {
            target_contract: self.enterprise_address.clone(),
            msg: serde_json::to_string(&Unstake(params))?,
        })
    }

    fn adapt_claim(&self, _: QueryContext) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(AdapterResponse {
            target_contract: self.enterprise_address.clone(),
            msg: serde_json::to_string(&Claim {})?,
        })
    }
}

impl EnterpriseFacadeV5 {
    fn query_enterprise_contract<T: DeserializeOwned>(
        &self,
        deps: Deps,
        msg: &impl Serialize,
    ) -> EnterpriseFacadeResult<T> {
        Ok(deps
            .querier
            .query_wasm_smart(self.enterprise_address.to_string(), &msg)?)
    }
}

/// This is what execute messages for Enterprise contract looked like for v5.
#[cw_serde]
enum ExecuteV5Msg {
    // facaded part
    ExecuteProposal(ExecuteProposalMsg),
    // adapted part
    CreateProposal(CreateProposalMsg),
    CreateCouncilProposal(CreateProposalMsg),
    CastVote(CastVoteMsg),
    CastCouncilVote(CastVoteMsg),
    Unstake(UnstakeMsg),
    Claim {},
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
