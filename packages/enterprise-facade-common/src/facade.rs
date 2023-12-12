use common::cw::QueryContext;
use enterprise_facade_api::api::{
    AdapterResponse, AssetWhitelistParams, AssetWhitelistResponse, CastVoteMsg, ClaimsParams,
    ClaimsResponse, CreateProposalMsg, CreateProposalWithDenomDepositMsg,
    CreateProposalWithTokenDepositMsg, DaoInfoResponse, ExecuteProposalMsg, ListMultisigMembersMsg,
    MemberInfoResponse, MemberVoteParams, MemberVoteResponse, MultisigMembersResponse,
    NftWhitelistParams, NftWhitelistResponse, ProposalParams, ProposalResponse,
    ProposalStatusParams, ProposalStatusResponse, ProposalVotesParams, ProposalVotesResponse,
    ProposalsParams, ProposalsResponse, QueryMemberInfoMsg, StakeMsg, StakedNftsParams,
    StakedNftsResponse, TotalStakedAmountResponse, TreasuryAddressResponse, UnstakeMsg,
    UserStakeParams, UserStakeResponse, V2MigrationStageResponse,
};
use enterprise_facade_api::error::EnterpriseFacadeResult;
use enterprise_governance_controller_api::api::CreateProposalWithNftDepositMsg;
use enterprise_outposts_api::api::{CrossChainTreasuriesParams, CrossChainTreasuriesResponse};
use enterprise_treasury_api::api::{
    HasIncompleteV2MigrationResponse, HasUnmovedStakesOrClaimsResponse,
};

pub trait EnterpriseFacade {
    fn query_treasury_address(
        &self,
        qctx: QueryContext,
    ) -> EnterpriseFacadeResult<TreasuryAddressResponse>;

    fn query_dao_info(&self, qctx: QueryContext) -> EnterpriseFacadeResult<DaoInfoResponse>;

    fn query_member_info(
        &self,
        qctx: QueryContext,
        msg: QueryMemberInfoMsg,
    ) -> EnterpriseFacadeResult<MemberInfoResponse>;

    fn query_list_multisig_members(
        &self,
        qctx: QueryContext,
        msg: ListMultisigMembersMsg,
    ) -> EnterpriseFacadeResult<MultisigMembersResponse>;

    fn query_asset_whitelist(
        &self,
        qctx: QueryContext,
        params: AssetWhitelistParams,
    ) -> EnterpriseFacadeResult<AssetWhitelistResponse>;

    fn query_nft_whitelist(
        &self,
        qctx: QueryContext,
        params: NftWhitelistParams,
    ) -> EnterpriseFacadeResult<NftWhitelistResponse>;

    fn query_proposal(
        &self,
        qctx: QueryContext,
        params: ProposalParams,
    ) -> EnterpriseFacadeResult<ProposalResponse>;

    fn query_proposals(
        &self,
        qctx: QueryContext,
        params: ProposalsParams,
    ) -> EnterpriseFacadeResult<ProposalsResponse>;

    fn query_proposal_status(
        &self,
        qctx: QueryContext,
        params: ProposalStatusParams,
    ) -> EnterpriseFacadeResult<ProposalStatusResponse>;

    fn query_member_vote(
        &self,
        qctx: QueryContext,
        params: MemberVoteParams,
    ) -> EnterpriseFacadeResult<MemberVoteResponse>;

    fn query_proposal_votes(
        &self,
        qctx: QueryContext,
        params: ProposalVotesParams,
    ) -> EnterpriseFacadeResult<ProposalVotesResponse>;

    fn query_user_stake(
        &self,
        qctx: QueryContext,
        params: UserStakeParams,
    ) -> EnterpriseFacadeResult<UserStakeResponse>;

    fn query_total_staked_amount(
        &self,
        qctx: QueryContext,
    ) -> EnterpriseFacadeResult<TotalStakedAmountResponse>;

    fn query_staked_nfts(
        &self,
        qctx: QueryContext,
        params: StakedNftsParams,
    ) -> EnterpriseFacadeResult<StakedNftsResponse>;

    fn query_claims(
        &self,
        qctx: QueryContext,
        params: ClaimsParams,
    ) -> EnterpriseFacadeResult<ClaimsResponse>;

    fn query_releasable_claims(
        &self,
        qctx: QueryContext,
        params: ClaimsParams,
    ) -> EnterpriseFacadeResult<ClaimsResponse>;

    fn query_cross_chain_treasuries(
        &self,
        qctx: QueryContext,
        params: CrossChainTreasuriesParams,
    ) -> EnterpriseFacadeResult<CrossChainTreasuriesResponse>;

    fn query_has_incomplete_v2_migration(
        &self,
        _: QueryContext,
    ) -> EnterpriseFacadeResult<HasIncompleteV2MigrationResponse>;

    fn query_has_unmoved_stakes_or_claims(
        &self,
        _: QueryContext,
    ) -> EnterpriseFacadeResult<HasUnmovedStakesOrClaimsResponse>;

    fn query_v2_migration_stage(
        &self,
        _: QueryContext,
    ) -> EnterpriseFacadeResult<V2MigrationStageResponse>;

    fn adapt_create_proposal(
        &self,
        qctx: QueryContext,
        params: CreateProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_create_proposal_with_denom_deposit(
        &self,
        qctx: QueryContext,
        params: CreateProposalWithDenomDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_create_proposal_with_token_deposit(
        &self,
        qctx: QueryContext,
        params: CreateProposalWithTokenDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_create_proposal_with_nft_deposit(
        &self,
        qctx: QueryContext,
        params: CreateProposalWithNftDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_create_council_proposal(
        &self,
        qctx: QueryContext,
        params: CreateProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_cast_vote(
        &self,
        qctx: QueryContext,
        params: CastVoteMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_cast_council_vote(
        &self,
        qctx: QueryContext,
        params: CastVoteMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_execute_proposal(
        &self,
        qctx: QueryContext,
        params: ExecuteProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_stake(
        &self,
        qctx: QueryContext,
        params: StakeMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_unstake(
        &self,
        qctx: QueryContext,
        params: UnstakeMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_claim(&self, qctx: QueryContext) -> EnterpriseFacadeResult<AdapterResponse>;
}
