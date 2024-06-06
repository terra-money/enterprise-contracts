use enterprise_facade_api::api::{
    AdapterResponse, AssetWhitelistParams, AssetWhitelistResponse, CastVoteMsg, ClaimsParams,
    ClaimsResponse, ComponentContractsResponse, CreateProposalMsg,
    CreateProposalWithDenomDepositMsg, CreateProposalWithTokenDepositMsg, DaoInfoResponse,
    ExecuteProposalMsg, ListMultisigMembersMsg, MemberInfoResponse, MemberVoteParams,
    MemberVoteResponse, MultisigMembersResponse, NftWhitelistParams, NftWhitelistResponse,
    NumberProposalsTrackedResponse, ProposalParams, ProposalResponse, ProposalStatusParams,
    ProposalStatusResponse, ProposalVotesParams, ProposalVotesResponse, ProposalsParams,
    ProposalsResponse, QueryMemberInfoMsg, StakeMsg, StakedNftsParams, StakedNftsResponse,
    TotalStakedAmountResponse, TreasuryAddressResponse, UnstakeMsg, UserStakeParams,
    UserStakeResponse, V2MigrationStageResponse,
};
use enterprise_facade_api::error::EnterpriseFacadeResult;
use enterprise_governance_controller_api::api::CreateProposalWithNftDepositMsg;
use enterprise_outposts_api::api::{CrossChainTreasuriesParams, CrossChainTreasuriesResponse};
use enterprise_treasury_api::api::{
    HasIncompleteV2MigrationResponse, HasUnmovedStakesOrClaimsResponse,
};
use membership_common_api::api::{MembersParams, MembersResponse};

pub trait EnterpriseFacade {
    fn query_treasury_address(&self) -> EnterpriseFacadeResult<TreasuryAddressResponse>;

    fn query_dao_info(&self) -> EnterpriseFacadeResult<DaoInfoResponse>;

    fn query_component_contracts(&self) -> EnterpriseFacadeResult<ComponentContractsResponse>;

    fn query_member_info(
        &self,
        msg: QueryMemberInfoMsg,
    ) -> EnterpriseFacadeResult<MemberInfoResponse>;

    fn query_members(&self, msg: MembersParams) -> EnterpriseFacadeResult<MembersResponse>;

    fn query_list_multisig_members(
        &self,
        msg: ListMultisigMembersMsg,
    ) -> EnterpriseFacadeResult<MultisigMembersResponse>;

    fn query_asset_whitelist(
        &self,
        params: AssetWhitelistParams,
    ) -> EnterpriseFacadeResult<AssetWhitelistResponse>;

    fn query_nft_whitelist(
        &self,
        params: NftWhitelistParams,
    ) -> EnterpriseFacadeResult<NftWhitelistResponse>;

    fn query_number_proposals_tracked(
        &self,
    ) -> EnterpriseFacadeResult<NumberProposalsTrackedResponse>;

    fn query_proposal(&self, params: ProposalParams) -> EnterpriseFacadeResult<ProposalResponse>;

    fn query_proposals(&self, params: ProposalsParams)
        -> EnterpriseFacadeResult<ProposalsResponse>;

    fn query_proposal_status(
        &self,
        params: ProposalStatusParams,
    ) -> EnterpriseFacadeResult<ProposalStatusResponse>;

    fn query_member_vote(
        &self,
        params: MemberVoteParams,
    ) -> EnterpriseFacadeResult<MemberVoteResponse>;

    fn query_proposal_votes(
        &self,
        params: ProposalVotesParams,
    ) -> EnterpriseFacadeResult<ProposalVotesResponse>;

    fn query_user_stake(
        &self,
        params: UserStakeParams,
    ) -> EnterpriseFacadeResult<UserStakeResponse>;

    fn query_total_staked_amount(&self) -> EnterpriseFacadeResult<TotalStakedAmountResponse>;

    fn query_staked_nfts(
        &self,
        params: StakedNftsParams,
    ) -> EnterpriseFacadeResult<StakedNftsResponse>;

    fn query_claims(&self, params: ClaimsParams) -> EnterpriseFacadeResult<ClaimsResponse>;

    fn query_releasable_claims(
        &self,
        params: ClaimsParams,
    ) -> EnterpriseFacadeResult<ClaimsResponse>;

    fn query_cross_chain_treasuries(
        &self,
        params: CrossChainTreasuriesParams,
    ) -> EnterpriseFacadeResult<CrossChainTreasuriesResponse>;

    fn query_has_incomplete_v2_migration(
        &self,
    ) -> EnterpriseFacadeResult<HasIncompleteV2MigrationResponse>;

    fn query_has_unmoved_stakes_or_claims(
        &self,
    ) -> EnterpriseFacadeResult<HasUnmovedStakesOrClaimsResponse>;

    fn query_v2_migration_stage(&self) -> EnterpriseFacadeResult<V2MigrationStageResponse>;

    fn adapt_create_proposal(
        &self,
        params: CreateProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_create_proposal_with_denom_deposit(
        &self,
        params: CreateProposalWithDenomDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_create_proposal_with_token_deposit(
        &self,
        params: CreateProposalWithTokenDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_create_proposal_with_nft_deposit(
        &self,
        params: CreateProposalWithNftDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_create_council_proposal(
        &self,
        params: CreateProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_cast_vote(&self, params: CastVoteMsg) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_cast_council_vote(
        &self,
        params: CastVoteMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_execute_proposal(
        &self,
        params: ExecuteProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_stake(&self, params: StakeMsg) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_unstake(&self, params: UnstakeMsg) -> EnterpriseFacadeResult<AdapterResponse>;

    fn adapt_claim(&self) -> EnterpriseFacadeResult<AdapterResponse>;
}
