use crate::helpers::ADDR_FACADE;
use cosmwasm_std::Addr;
use cw_multi_test::App;
use enterprise_facade_api::api::{
    AdapterResponse, AssetWhitelistParams, AssetWhitelistResponse, CastVoteMsg, ClaimsParams,
    ClaimsResponse, ComponentContractsResponse, CreateProposalMsg,
    CreateProposalWithDenomDepositMsg, CreateProposalWithTokenDepositMsg, DaoInfoResponse,
    DaoMetadata, ExecuteProposalMsg, GovConfigFacade, ListMultisigMembersMsg, Logo,
    MemberInfoResponse, MemberVoteParams, MemberVoteResponse, MultisigMembersResponse,
    NftWhitelistParams, NftWhitelistResponse, ProposalParams, ProposalResponse,
    ProposalStatusParams, ProposalStatusResponse, ProposalVotesParams, ProposalVotesResponse,
    ProposalsParams, ProposalsResponse, QueryMemberInfoMsg, StakeMsg, StakedNftsParams,
    StakedNftsResponse, TotalStakedAmountResponse, TreasuryAddressResponse, UnstakeMsg,
    UserStakeParams, UserStakeResponse, V2MigrationStageResponse,
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
use enterprise_governance_controller_api::api::{CreateProposalWithNftDepositMsg, GovConfig};
use enterprise_outposts_api::api::CrossChainTreasuriesParams;
use enterprise_treasury_api::api::{
    HasIncompleteV2MigrationResponse, HasUnmovedStakesOrClaimsResponse,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

// Helper implementation of the Enterprise facade to use in the tests
pub struct TestFacade<'a> {
    pub app: &'a App,
    pub dao_addr: Addr,
}

impl EnterpriseFacade for TestFacade<'_> {
    fn query_treasury_address(&self) -> EnterpriseFacadeResult<TreasuryAddressResponse> {
        self.query_facade(&TreasuryAddress {
            contract: self.dao_addr.clone(),
        })
    }

    fn query_dao_info(&self) -> EnterpriseFacadeResult<DaoInfoResponse> {
        self.query_facade(&DaoInfo {
            contract: self.dao_addr.clone(),
        })
    }

    fn query_component_contracts(&self) -> EnterpriseFacadeResult<ComponentContractsResponse> {
        self.query_facade(&ComponentContracts {
            contract: self.dao_addr.clone(),
        })
    }

    fn query_member_info(
        &self,
        msg: QueryMemberInfoMsg,
    ) -> EnterpriseFacadeResult<MemberInfoResponse> {
        self.query_facade(&MemberInfo {
            contract: self.dao_addr.clone(),
            msg,
        })
    }

    fn query_list_multisig_members(
        &self,
        msg: ListMultisigMembersMsg,
    ) -> EnterpriseFacadeResult<MultisigMembersResponse> {
        self.query_facade(&ListMultisigMembers {
            contract: self.dao_addr.clone(),
            msg,
        })
    }

    fn query_asset_whitelist(
        &self,
        params: AssetWhitelistParams,
    ) -> EnterpriseFacadeResult<AssetWhitelistResponse> {
        self.query_facade(&AssetWhitelist {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_nft_whitelist(
        &self,
        params: NftWhitelistParams,
    ) -> EnterpriseFacadeResult<NftWhitelistResponse> {
        self.query_facade(&NftWhitelist {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_proposal(&self, params: ProposalParams) -> EnterpriseFacadeResult<ProposalResponse> {
        self.query_facade(&Proposal {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_proposals(
        &self,
        params: ProposalsParams,
    ) -> EnterpriseFacadeResult<ProposalsResponse> {
        self.query_facade(&Proposals {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_proposal_status(
        &self,
        params: ProposalStatusParams,
    ) -> EnterpriseFacadeResult<ProposalStatusResponse> {
        self.query_facade(&ProposalStatus {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_member_vote(
        &self,
        params: MemberVoteParams,
    ) -> EnterpriseFacadeResult<MemberVoteResponse> {
        self.query_facade(&MemberVote {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_proposal_votes(
        &self,
        params: ProposalVotesParams,
    ) -> EnterpriseFacadeResult<ProposalVotesResponse> {
        self.query_facade(&ProposalVotes {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_user_stake(
        &self,
        params: UserStakeParams,
    ) -> EnterpriseFacadeResult<UserStakeResponse> {
        self.query_facade(&UserStake {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_total_staked_amount(&self) -> EnterpriseFacadeResult<TotalStakedAmountResponse> {
        self.query_facade(&TotalStakedAmount {
            contract: self.dao_addr.clone(),
        })
    }

    fn query_staked_nfts(
        &self,
        params: StakedNftsParams,
    ) -> EnterpriseFacadeResult<StakedNftsResponse> {
        self.query_facade(&StakedNfts {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_claims(&self, params: ClaimsParams) -> EnterpriseFacadeResult<ClaimsResponse> {
        self.query_facade(&Claims {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_releasable_claims(
        &self,
        params: ClaimsParams,
    ) -> EnterpriseFacadeResult<ClaimsResponse> {
        self.query_facade(&ReleasableClaims {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_cross_chain_treasuries(
        &self,
        params: CrossChainTreasuriesParams,
    ) -> EnterpriseFacadeResult<enterprise_outposts_api::api::CrossChainTreasuriesResponse> {
        self.query_facade(&CrossChainTreasuries {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn query_has_incomplete_v2_migration(
        &self,
    ) -> EnterpriseFacadeResult<HasIncompleteV2MigrationResponse> {
        self.query_facade(&HasIncompleteV2Migration {
            contract: self.dao_addr.clone(),
        })
    }

    fn query_has_unmoved_stakes_or_claims(
        &self,
    ) -> EnterpriseFacadeResult<HasUnmovedStakesOrClaimsResponse> {
        self.query_facade(&HasUnmovedStakesOrClaims {
            contract: self.dao_addr.clone(),
        })
    }

    fn query_v2_migration_stage(&self) -> EnterpriseFacadeResult<V2MigrationStageResponse> {
        self.query_facade(&V2MigrationStage {
            contract: self.dao_addr.clone(),
        })
    }

    fn adapt_create_proposal(
        &self,
        params: CreateProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&CreateProposalAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_create_proposal_with_denom_deposit(
        &self,
        params: CreateProposalWithDenomDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&CreateProposalWithDenomDepositAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_create_proposal_with_token_deposit(
        &self,
        params: CreateProposalWithTokenDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&CreateProposalWithTokenDepositAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_create_proposal_with_nft_deposit(
        &self,
        params: CreateProposalWithNftDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&CreateProposalWithNftDepositAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_create_council_proposal(
        &self,
        params: CreateProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&CreateCouncilProposalAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_cast_vote(&self, params: CastVoteMsg) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&CastVoteAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_cast_council_vote(
        &self,
        params: CastVoteMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&CastCouncilVoteAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_execute_proposal(
        &self,
        params: ExecuteProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&ExecuteProposalAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_stake(&self, params: StakeMsg) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&StakeAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_unstake(&self, params: UnstakeMsg) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&UnstakeAdapted {
            contract: self.dao_addr.clone(),
            params,
        })
    }

    fn adapt_claim(&self) -> EnterpriseFacadeResult<AdapterResponse> {
        self.query_facade(&ClaimAdapted {
            contract: self.dao_addr.clone(),
        })
    }
}

impl TestFacade<'_> {
    fn query_facade<T: DeserializeOwned>(&self, msg: &impl Serialize) -> EnterpriseFacadeResult<T> {
        self.app
            .wrap()
            .query_wasm_smart(ADDR_FACADE, msg)
            .map_err(|e| EnterpriseFacadeError::Std(e))
    }
}

pub fn from_facade_metadata(metadata: DaoMetadata) -> enterprise_protocol::api::DaoMetadata {
    let logo = match metadata.logo {
        Logo::Url(url) => enterprise_protocol::api::Logo::Url(url),
        Logo::None => enterprise_protocol::api::Logo::None,
    };
    enterprise_protocol::api::DaoMetadata {
        name: metadata.name,
        description: metadata.description,
        logo,
        socials: enterprise_protocol::api::DaoSocialData {
            github_username: metadata.socials.github_username,
            discord_username: metadata.socials.discord_username,
            twitter_username: metadata.socials.twitter_username,
            telegram_username: metadata.socials.telegram_username,
        },
    }
}

pub fn from_facade_gov_config(gov_config: GovConfigFacade) -> GovConfig {
    GovConfig {
        quorum: gov_config.quorum,
        threshold: gov_config.threshold,
        veto_threshold: Some(gov_config.veto_threshold),
        vote_duration: gov_config.vote_duration,
        minimum_deposit: gov_config.minimum_deposit,
        allow_early_proposal_execution: gov_config.allow_early_proposal_execution,
    }
}
