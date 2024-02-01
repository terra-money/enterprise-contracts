use crate::helpers::cw_multitest_helpers::ADDR_FACADE;
use crate::helpers::funds_distributor_helpers::TestFundsDistributorContract;
use crate::helpers::membership_helpers::TestMembershipContract;
use crate::traits::IntoAddr;
use cosmwasm_std::{Addr, Uint128};
use cw_asset::AssetInfo;
use cw_multi_test::App;
use denom_staking_api::api::DenomConfigResponse;
use denom_staking_api::msg::QueryMsg::DenomConfig;
use enterprise_facade_api::api::{
    AdapterResponse, AssetWhitelistParams, AssetWhitelistResponse, CastVoteMsg, ClaimsParams,
    ClaimsResponse, ComponentContractsResponse, CreateProposalMsg,
    CreateProposalWithDenomDepositMsg, CreateProposalWithTokenDepositMsg, DaoCouncil,
    DaoInfoResponse, DaoMetadata, DaoType, ExecuteProposalMsg, GovConfigFacade,
    ListMultisigMembersMsg, Logo, MemberInfoResponse, MemberVoteParams, MemberVoteResponse,
    MultisigMember, MultisigMembersResponse, NftWhitelistParams, NftWhitelistResponse,
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
    MemberVote, Members, NftWhitelist, Proposal, ProposalStatus, ProposalVotes, Proposals,
    ReleasableClaims, StakeAdapted, StakedNfts, TotalStakedAmount, TreasuryAddress, UnstakeAdapted,
    UserStake, V2MigrationStage,
};
use enterprise_facade_common::facade::EnterpriseFacade;
use enterprise_governance_controller_api::api::{
    CreateProposalWithNftDepositMsg, DaoCouncilSpec, GovConfig,
};
use enterprise_outposts_api::api::CrossChainTreasuriesParams;
use enterprise_treasury_api::api::{
    HasIncompleteV2MigrationResponse, HasUnmovedStakesOrClaimsResponse,
};
use membership_common_api::api::{MembersParams, MembersResponse};
use nft_staking_api::api::NftConfigResponse;
use nft_staking_api::msg::QueryMsg::NftConfig;
use serde::de::DeserializeOwned;
use serde::Serialize;
use token_staking_api::api::TokenConfigResponse;
use token_staking_api::msg::QueryMsg::TokenConfig;

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

    fn query_members(&self, msg: MembersParams) -> EnterpriseFacadeResult<MembersResponse> {
        self.query_facade(&Members {
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

// convenience functions
impl TestFacade<'_> {
    pub fn member_info(
        &self,
        member: impl Into<String>,
    ) -> EnterpriseFacadeResult<MemberInfoResponse> {
        self.query_member_info(QueryMemberInfoMsg {
            member_address: member.into(),
        })
    }
}

// assertion helpers
impl TestFacade<'_> {
    pub fn assert_multisig_members_list(
        &self,
        start_after: Option<&str>,
        limit: Option<u32>,
        members: Vec<(impl Into<String>, u8)>,
    ) {
        let members_list = self
            .query_list_multisig_members(ListMultisigMembersMsg {
                start_after: start_after.map(|it| it.to_string()),
                limit,
            })
            .unwrap();
        assert_eq!(
            members_list.members,
            members
                .into_iter()
                .map(|(user, weight)| MultisigMember {
                    address: user.into(),
                    weight: weight.into()
                })
                .collect::<Vec<MultisigMember>>(),
        );
    }

    pub fn assert_total_staked(&self, total_staked: u8) {
        let total_staked_amount = self
            .query_total_staked_amount()
            .unwrap()
            .total_staked_amount;
        assert_eq!(total_staked_amount, Uint128::from(total_staked));
    }

    pub fn assert_asset_whitelist(&self, assets: Vec<AssetInfo>) {
        let asset_whitelist = self
            .query_asset_whitelist(AssetWhitelistParams {
                start_after: None,
                limit: None,
            })
            .unwrap()
            .assets;
        assert_eq!(asset_whitelist, assets);
    }

    pub fn assert_nft_whitelist(&self, nfts: Vec<&str>) {
        let nft_whitelist = self
            .query_nft_whitelist(NftWhitelistParams {
                start_after: None,
                limit: None,
            })
            .unwrap()
            .nfts;
        assert_eq!(
            nft_whitelist,
            nfts.into_iter()
                .map(|it| it.into_addr())
                .collect::<Vec<Addr>>()
        );
    }

    pub fn assert_dao_type(&self, dao_type: DaoType) {
        assert_eq!(self.query_dao_info().unwrap().dao_type, dao_type)
    }
}

impl<'a> TestFacade<'a> {
    pub fn membership(&self) -> TestMembershipContract<'a> {
        TestMembershipContract {
            app: self.app,
            addr: self.components().membership_contract.unwrap(),
        }
    }

    pub fn council_membership(&self) -> TestMembershipContract<'a> {
        TestMembershipContract {
            app: self.app,
            addr: self.components().council_membership_contract.unwrap(),
        }
    }

    pub fn funds_distributor(&self) -> TestFundsDistributorContract<'a> {
        TestFundsDistributorContract {
            app: self.app,
            addr: self.components().funds_distributor_contract,
        }
    }

    pub fn factory_addr(&self) -> Addr {
        self.components().enterprise_factory_contract
    }

    pub fn enterprise_addr(&self) -> Addr {
        self.components().enterprise_contract
    }

    pub fn funds_distributor_addr(&self) -> Addr {
        self.components().funds_distributor_contract
    }

    pub fn governance_addr(&self) -> Addr {
        self.components().enterprise_governance_contract.unwrap()
    }

    pub fn gov_controller_addr(&self) -> Addr {
        self.components()
            .enterprise_governance_controller_contract
            .unwrap()
    }

    pub fn outposts_addr(&self) -> Addr {
        self.components().enterprise_outposts_contract.unwrap()
    }

    pub fn treasury_addr(&self) -> Addr {
        self.components().enterprise_treasury_contract.unwrap()
    }

    pub fn membership_addr(&self) -> Addr {
        self.components().membership_contract.unwrap()
    }

    pub fn council_membership_addr(&self) -> Addr {
        self.components().council_membership_contract.unwrap()
    }

    pub fn attestation_addr(&self) -> Addr {
        self.components().attestation_contract.unwrap()
    }

    fn components(&self) -> ComponentContractsResponse {
        self.query_component_contracts().unwrap()
    }
}

impl TestFacade<'_> {
    pub fn token_config(&self) -> EnterpriseFacadeResult<TokenConfigResponse> {
        let membership_contract = self
            .query_component_contracts()?
            .membership_contract
            .unwrap();
        let token_config = self
            .app
            .wrap()
            .query_wasm_smart(membership_contract.to_string(), &TokenConfig {})?;
        Ok(token_config)
    }

    pub fn nft_config(&self) -> EnterpriseFacadeResult<NftConfigResponse> {
        let membership_contract = self
            .query_component_contracts()?
            .membership_contract
            .unwrap();
        let nft_config = self
            .app
            .wrap()
            .query_wasm_smart(membership_contract.to_string(), &NftConfig {})?;
        Ok(nft_config)
    }

    pub fn denom_config(&self) -> EnterpriseFacadeResult<DenomConfigResponse> {
        let membership_contract = self
            .query_component_contracts()?
            .membership_contract
            .unwrap();
        let denom_config = self
            .app
            .wrap()
            .query_wasm_smart(membership_contract.to_string(), &DenomConfig {})?;
        Ok(denom_config)
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

pub fn from_facade_dao_council(council: DaoCouncil) -> DaoCouncilSpec {
    DaoCouncilSpec {
        members: council.members.into_iter().map(Addr::into_string).collect(),
        quorum: council.quorum,
        threshold: council.threshold,
        allowed_proposal_action_types: council
            .allowed_proposal_action_types
            .into_iter()
            .map(|it| it.into())
            .collect(),
    }
}
