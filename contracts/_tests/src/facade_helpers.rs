use crate::helpers::ADDR_FACADE;
use common::cw::QueryContext;
use cosmwasm_std::Addr;
use cw_multi_test::App;
use enterprise_facade_api::api::{
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
use enterprise_facade_api::error::{EnterpriseFacadeError, EnterpriseFacadeResult};
use enterprise_facade_api::msg::QueryMsg::{
    AssetWhitelist, CastCouncilVoteAdapted, CastVoteAdapted, ClaimAdapted, Claims,
    ComponentContracts, CreateCouncilProposalAdapted, CreateProposalAdapted,
    CreateProposalWithDenomDepositAdapted, CreateProposalWithNftDepositAdapted,
    CreateProposalWithTokenDepositAdapted, CrossChainTreasuries, DaoInfo, ExecuteProposalAdapted,
    HasIncompleteV2Migration, HasUnmovedStakesOrClaims, ListMultisigMembers, MemberInfo,
    MemberVote, NftWhitelist, Proposal, ProposalStatus, ProposalVotes, Proposals, ReleasableClaims,
    StakeAdapted, StakedNfts, TotalStakedAmount, TreasuryAddress, UnstakeAdapted, UserStake,
    V2MigrationStage,
};
use enterprise_facade_common::facade::EnterpriseFacade;
use enterprise_governance_controller_api::api::CreateProposalWithNftDepositMsg;
use enterprise_outposts_api::api::CrossChainTreasuriesParams;
use enterprise_treasury_api::api::{
    HasIncompleteV2MigrationResponse, HasUnmovedStakesOrClaimsResponse,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct TestFacade {
    pub app: App,
    pub dao_addr: Addr,
}

impl EnterpriseFacade for TestFacade {
    fn query_treasury_address(
        &self,
        _qctx: QueryContext,
    ) -> EnterpriseFacadeResult<TreasuryAddressResponse> {
        self.query_facade(&TreasuryAddress {
            contract: self.dao_addr.clone(),
        })
    }

    fn query_dao_info(&self, _qctx: QueryContext) -> EnterpriseFacadeResult<DaoInfoResponse> {
        self.query_facade(&DaoInfo {
            contract: self.dao_addr.clone(),
        })
    }

    fn query_component_contracts(
        &self,
        _qctx: QueryContext,
    ) -> EnterpriseFacadeResult<ComponentContractsResponse> {
        self.query_facade(&ComponentContracts {
            contract: self.dao_addr.clone(),
        })
    }

    fn query_member_info(
        &self,
        _qctx: QueryContext,
        msg: QueryMemberInfoMsg,
    ) -> EnterpriseFacadeResult<MemberInfoResponse> {
        self.query_facade(&MemberInfo {
            contract: self.dao_addr.clone(),
            msg,
        })
    }

    fn query_list_multisig_members(
        &self,
        _qctx: QueryContext,
        msg: ListMultisigMembersMsg,
    ) -> EnterpriseFacadeResult<MultisigMembersResponse> {
        self.query_facade(&ListMultisigMembers {
            contract: self.dao_addr.clone(),
            msg,
        })
    }

    fn query_asset_whitelist(
        &self,
        _qctx: QueryContext,
        params: AssetWhitelistParams,
    ) -> EnterpriseFacadeResult<AssetWhitelistResponse> {
        self.query_facade(&AssetWhitelist {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_nft_whitelist(
        &self,
        _qctx: QueryContext,
        params: NftWhitelistParams,
    ) -> EnterpriseFacadeResult<NftWhitelistResponse> {
        self.query_facade(&NftWhitelist {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_proposal(
        &self,
        _qctx: QueryContext,
        params: ProposalParams,
    ) -> EnterpriseFacadeResult<ProposalResponse> {
        self.query_facade(&Proposal {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_proposals(
        &self,
        _qctx: QueryContext,
        params: ProposalsParams,
    ) -> EnterpriseFacadeResult<ProposalsResponse> {
        self.query_facade(&Proposals {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_proposal_status(
        &self,
        _qctx: QueryContext,
        params: ProposalStatusParams,
    ) -> EnterpriseFacadeResult<ProposalStatusResponse> {
        self.query_facade(&ProposalStatus {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_member_vote(
        &self,
        _qctx: QueryContext,
        params: MemberVoteParams,
    ) -> EnterpriseFacadeResult<MemberVoteResponse> {
        self.query_facade(&MemberVote {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_proposal_votes(
        &self,
        _qctx: QueryContext,
        params: ProposalVotesParams,
    ) -> EnterpriseFacadeResult<ProposalVotesResponse> {
        self.query_facade(&ProposalVotes {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_user_stake(
        &self,
        _qctx: QueryContext,
        params: UserStakeParams,
    ) -> EnterpriseFacadeResult<UserStakeResponse> {
        self.query_facade(&UserStake {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_total_staked_amount(
        &self,
        _qctx: QueryContext,
    ) -> EnterpriseFacadeResult<TotalStakedAmountResponse> {
        self.query_facade(&TotalStakedAmount {
            contract: self.dao_addr.clone(),
        })
    }

    fn query_staked_nfts(
        &self,
        _qctx: QueryContext,
        params: StakedNftsParams,
    ) -> EnterpriseFacadeResult<StakedNftsResponse> {
        self.query_facade(&StakedNfts {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_claims(
        &self,
        _qctx: QueryContext,
        params: ClaimsParams,
    ) -> EnterpriseFacadeResult<ClaimsResponse> {
        self.query_facade(&Claims {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_releasable_claims(
        &self,
        _qctx: QueryContext,
        params: ClaimsParams,
    ) -> EnterpriseFacadeResult<ClaimsResponse> {
        self.query_facade(&ReleasableClaims {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_cross_chain_treasuries(
        &self,
        _qctx: QueryContext,
        params: CrossChainTreasuriesParams,
    ) -> EnterpriseFacadeResult<enterprise_outposts_api::api::CrossChainTreasuriesResponse> {
        self.query_facade(&CrossChainTreasuries {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_has_incomplete_v2_migration(
        &self,
        _: QueryContext,
    ) -> EnterpriseFacadeResult<HasIncompleteV2MigrationResponse> {
        self.query_facade(&HasIncompleteV2Migration {
            contract: self.dao_addr.clone(),
        })
    }

    fn query_has_unmoved_stakes_or_claims(
        &self,
        _: QueryContext,
    ) -> EnterpriseFacadeResult<HasUnmovedStakesOrClaimsResponse> {
        self.query_facade(&HasUnmovedStakesOrClaims {
            contract: self.dao_addr.clone(),
        })
    }

    fn query_v2_migration_stage(
        &self,
        _: QueryContext,
    ) -> EnterpriseFacadeResult<V2MigrationStageResponse> {
        self.query_facade(&V2MigrationStage {
            contract: self.dao_addr.clone(),
        })
    }

    fn adapt_create_proposal(
        &self,
        _qctx: QueryContext,
        params: CreateProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&CreateProposalAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_create_proposal_with_denom_deposit(
        &self,
        _qctx: QueryContext,
        params: CreateProposalWithDenomDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&CreateProposalWithDenomDepositAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_create_proposal_with_token_deposit(
        &self,
        _qctx: QueryContext,
        params: CreateProposalWithTokenDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&CreateProposalWithTokenDepositAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_create_proposal_with_nft_deposit(
        &self,
        _qctx: QueryContext,
        params: CreateProposalWithNftDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&CreateProposalWithNftDepositAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_create_council_proposal(
        &self,
        _qctx: QueryContext,
        params: CreateProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&CreateCouncilProposalAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_cast_vote(
        &self,
        _qctx: QueryContext,
        params: CastVoteMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&CastVoteAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_cast_council_vote(
        &self,
        _qctx: QueryContext,
        params: CastVoteMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&CastCouncilVoteAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_execute_proposal(
        &self,
        _qctx: QueryContext,
        params: ExecuteProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&ExecuteProposalAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_stake(
        &self,
        _qctx: QueryContext,
        params: StakeMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&StakeAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_unstake(
        &self,
        _qctx: QueryContext,
        params: UnstakeMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&UnstakeAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_claim(&self, _qctx: QueryContext) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&ClaimAdapted {
            contract: self.dao_addr.clone(),
        })
    }
}

impl TestFacade {
    fn query_facade<T: DeserializeOwned>(&self, msg: &impl Serialize) -> EnterpriseFacadeResult<T> {
        self.app
            .wrap()
            .query_wasm_smart(ADDR_FACADE, msg)
            .map_err(|e| EnterpriseFacadeError::Std(e))
    }
}
