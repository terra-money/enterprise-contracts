use crate::facade_post_rewrite::EnterpriseFacadePostRewrite;
use crate::facade_v5::EnterpriseFacadeV5;
use crate::facade_v5::QueryV5Msg::DaoInfo;
use common::cw::{Context, QueryContext};
use cosmwasm_std::{Addr, Deps, Response, StdResult};
use enterprise_facade_api::api::{
    AdapterResponse, AssetWhitelistParams, AssetWhitelistResponse, CastVoteMsg, ClaimsParams,
    ClaimsResponse, CreateProposalMsg, CreateProposalWithDenomDepositMsg,
    CreateProposalWithTokenDepositMsg, DaoInfoResponse, ExecuteProposalMsg, ListMultisigMembersMsg,
    MemberInfoResponse, MemberVoteParams, MemberVoteResponse, MultisigMembersResponse,
    NftWhitelistParams, NftWhitelistResponse, ProposalParams, ProposalResponse,
    ProposalStatusParams, ProposalStatusResponse, ProposalVotesParams, ProposalVotesResponse,
    ProposalsParams, ProposalsResponse, QueryMemberInfoMsg, StakeMsg, StakedNftsParams,
    StakedNftsResponse, TotalStakedAmountResponse, UnstakeMsg, UserStakeParams, UserStakeResponse,
};
use enterprise_facade_api::error::EnterpriseFacadeError::CannotCreateFacade;
use enterprise_facade_api::error::EnterpriseFacadeResult;
use enterprise_governance_controller_api::api::CreateProposalWithNftDepositMsg;
use enterprise_treasury_api::api::ConfigResponse;
use enterprise_treasury_api::msg::QueryMsg::Config;

pub trait EnterpriseFacade {
    fn execute_proposal(
        &self,
        ctx: &mut Context,
        msg: ExecuteProposalMsg,
    ) -> EnterpriseFacadeResult<Response>;

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

/// Get the correct facade implementation for the given address.
/// Address given will be for different contracts depending on Enterprise version.
/// For v0.5.0 (pre-rewrite) Enterprise, the address will be that of the enterprise contract itself.
/// For v1.0.0 (post-rewrite) Enterprise, the address will be that of the enterprise treasury contract.
pub fn get_facade(deps: Deps, address: Addr) -> EnterpriseFacadeResult<Box<dyn EnterpriseFacade>> {
    // attempt to query for DAO info
    let dao_info: StdResult<DaoInfoResponse> = deps
        .querier
        .query_wasm_smart(address.to_string(), &DaoInfo {});

    if dao_info.is_ok() {
        // if the query was successful, then this is a v0.5.0 (pre-rewrite) Enterprise contract
        Ok(Box::new(EnterpriseFacadeV5 {
            enterprise_address: address,
        }))
    } else {
        // if the query failed, this should be the post-rewrite Enterprise treasury, but let's check
        let treasury_config: ConfigResponse = deps
            .querier
            .query_wasm_smart(address.to_string(), &Config {})
            .map_err(|_| CannotCreateFacade)?;

        let governance_controller_config: enterprise_governance_controller_api::api::ConfigResponse = deps
            .querier
            .query_wasm_smart(treasury_config.admin.to_string(), &enterprise_governance_controller_api::msg::QueryMsg::Config {})
            .map_err(|_| CannotCreateFacade)?;

        Ok(Box::new(EnterpriseFacadePostRewrite {
            enterprise_treasury_address: address,
            enterprise_address: governance_controller_config.enterprise_contract,
        }))
    }
}
