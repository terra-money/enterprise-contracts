use common::cw::QueryContext;
use cosmwasm_std::{coins, to_json_binary, Addr, Decimal, Deps, Uint128, Uint64};
use cw721::Cw721ExecuteMsg::Approve;
use cw_utils::Duration;
use cw_utils::Expiration::Never;
use denom_staking_api::api::DenomConfigResponse;
use denom_staking_api::msg::QueryMsg::DenomConfig;
use enterprise_facade_api::api::{
    adapter_response_single_execute_msg, AdaptedBankMsg, AdaptedExecuteMsg, AdaptedMsg,
    AdapterResponse, AssetWhitelistParams, AssetWhitelistResponse, CastVoteMsg, Claim, ClaimAsset,
    ClaimsParams, ClaimsResponse, CreateProposalMsg, CreateProposalWithDenomDepositMsg,
    CreateProposalWithTokenDepositMsg, Cw20ClaimAsset, Cw721ClaimAsset, DaoCouncil,
    DaoInfoResponse, DaoMetadata, DaoSocialData, DaoType, DenomClaimAsset, DenomUserStake,
    ExecuteProposalMsg, GovConfigFacade, ListMultisigMembersMsg, MemberInfoResponse,
    MemberVoteParams, MemberVoteResponse, MultisigMember, MultisigMembersResponse, NftUserStake,
    NftWhitelistParams, NftWhitelistResponse, Proposal, ProposalParams, ProposalResponse,
    ProposalStatus, ProposalStatusFilter, ProposalStatusParams, ProposalStatusResponse,
    ProposalType, ProposalVotesParams, ProposalVotesResponse, ProposalsParams, ProposalsResponse,
    QueryMemberInfoMsg, StakeMsg, StakedNftsParams, StakedNftsResponse, TokenUserStake,
    TotalStakedAmountResponse, TreasuryAddressResponse, UnstakeMsg, UserStake, UserStakeParams,
    UserStakeResponse,
};
use enterprise_facade_api::error::DaoError::UnsupportedOperationForDaoType;
use enterprise_facade_api::error::EnterpriseFacadeError::Dao;
use enterprise_facade_api::error::EnterpriseFacadeResult;
use enterprise_facade_common::facade::EnterpriseFacade;
use enterprise_governance_controller_api::api::{
    CreateProposalWithNftDepositMsg, GovConfigResponse, ProposalAction,
};
use enterprise_governance_controller_api::msg::ExecuteMsg::{
    CreateProposal, CreateProposalWithNftDeposit, ExecuteProposal,
};
use enterprise_outposts_api::api::{CrossChainTreasuriesParams, CrossChainTreasuriesResponse};
use enterprise_outposts_api::msg::QueryMsg::CrossChainTreasuries;
use enterprise_protocol::api::ComponentContractsResponse;
use enterprise_protocol::msg::QueryMsg::{ComponentContracts, DaoInfo};
use enterprise_treasury_api::api::HasIncompleteV2MigrationResponse;
use enterprise_treasury_api::msg::QueryMsg::{
    AssetWhitelist, HasIncompleteV2Migration, NftWhitelist,
};
use membership_common_api::api::{
    MembersParams, MembersResponse, TotalWeightParams, TotalWeightResponse, UserWeightParams,
    UserWeightResponse,
};
use membership_common_api::msg::QueryMsg::{Members, TotalWeight, UserWeight};
use nft_staking_api::api::{NftConfigResponse, UserNftStakeParams, UserNftStakeResponse};
use nft_staking_api::msg::QueryMsg::{NftConfig, StakedNfts};
use token_staking_api::api::TokenConfigResponse;
use token_staking_api::msg::QueryMsg::TokenConfig;

/// Facade implementation for v1.0.0 of Enterprise contracts (post-contract-rewrite), i.e. DAO v2.
pub struct EnterpriseFacadeV2 {
    pub enterprise_address: Addr,
}

impl EnterpriseFacade for EnterpriseFacadeV2 {
    fn query_treasury_address(
        &self,
        qctx: QueryContext,
    ) -> EnterpriseFacadeResult<TreasuryAddressResponse> {
        let treasury_address = self
            .component_contracts(qctx.deps)?
            .enterprise_treasury_contract;

        Ok(TreasuryAddressResponse { treasury_address })
    }

    fn query_dao_info(&self, qctx: QueryContext) -> EnterpriseFacadeResult<DaoInfoResponse> {
        let dao_info: enterprise_protocol::api::DaoInfoResponse = qctx
            .deps
            .querier
            .query_wasm_smart(self.enterprise_address.to_string(), &DaoInfo {})?;

        let dao_type = map_dao_type(dao_info.dao_type);

        let component_contracts = self.component_contracts(qctx.deps)?;

        let gov_config: GovConfigResponse = qctx.deps.querier.query_wasm_smart(
            component_contracts
                .enterprise_governance_controller_contract
                .to_string(),
            &enterprise_governance_controller_api::msg::QueryMsg::GovConfig {},
        )?;

        // get the membership contract, which actually used to mean the CW20 / CW721 contract, not the new membership contracts
        let (dao_membership_contract, unlocking_period) = match dao_type {
            DaoType::Denom => {
                let denom_config: DenomConfigResponse = qctx.deps.querier.query_wasm_smart(
                    gov_config.dao_membership_contract.to_string(),
                    &DenomConfig {},
                )?;
                (denom_config.denom, denom_config.unlocking_period)
            }
            DaoType::Token => {
                let token_config: TokenConfigResponse = qctx.deps.querier.query_wasm_smart(
                    gov_config.dao_membership_contract.to_string(),
                    &TokenConfig {},
                )?;
                (
                    token_config.token_contract.to_string(),
                    token_config.unlocking_period,
                )
            }
            DaoType::Nft => {
                let nft_config: NftConfigResponse = qctx.deps.querier.query_wasm_smart(
                    gov_config.dao_membership_contract.to_string(),
                    &NftConfig {},
                )?;
                (
                    nft_config.nft_contract.to_string(),
                    nft_config.unlocking_period,
                )
            }
            DaoType::Multisig => {
                // doesn't make too much sense, but kept for backwards-compatibility since this was the previous behavior
                (self.enterprise_address.to_string(), Duration::Time(0))
            }
        };

        let council_members_response: MembersResponse = qctx.deps.querier.query_wasm_smart(
            gov_config.dao_council_membership_contract.to_string(),
            &Members(MembersParams {
                start_after: None,
                limit: Some(1000u32),
            }),
        )?;
        let council_members = council_members_response
            .members
            .into_iter()
            .map(|user_weight| user_weight.user)
            .collect();

        let dao_council = gov_config.council_gov_config.map(|config| DaoCouncil {
            members: council_members,
            allowed_proposal_action_types: config.allowed_proposal_action_types,
            quorum: config.quorum,
            threshold: config.threshold,
        });

        // Map DAO version to version code, formula: 100*100*(major) + 100*(minor) + (patch)
        let version = dao_info.dao_version;
        let major_component = Uint64::from(version.major).checked_mul(10_000u64.into())?;
        let minor_component = Uint64::from(version.minor).checked_mul(100u64.into())?;
        let patch_component = Uint64::from(version.patch);
        let dao_code_version = major_component
            .checked_add(minor_component)?
            .checked_add(patch_component)?;

        let veto_threshold = gov_config
            .gov_config
            .veto_threshold
            .unwrap_or(gov_config.gov_config.threshold);

        let gov_config = GovConfigFacade {
            quorum: gov_config.gov_config.quorum,
            threshold: gov_config.gov_config.threshold,
            veto_threshold,
            vote_duration: gov_config.gov_config.vote_duration,
            unlocking_period,
            minimum_deposit: gov_config.gov_config.minimum_deposit,
            allow_early_proposal_execution: gov_config.gov_config.allow_early_proposal_execution,
        };

        Ok(DaoInfoResponse {
            creation_date: dao_info.creation_date,
            metadata: DaoMetadata {
                name: dao_info.metadata.name,
                description: dao_info.metadata.description,
                logo: dao_info.metadata.logo.into(),
                socials: DaoSocialData {
                    github_username: dao_info.metadata.socials.github_username,
                    discord_username: dao_info.metadata.socials.discord_username,
                    twitter_username: dao_info.metadata.socials.twitter_username,
                    telegram_username: dao_info.metadata.socials.telegram_username,
                },
            },
            gov_config,
            dao_council,
            dao_type,
            dao_membership_contract,
            enterprise_factory_contract: component_contracts.enterprise_factory_contract,
            funds_distributor_contract: component_contracts.funds_distributor_contract,
            dao_code_version,
            dao_version: version,
        })
    }

    fn query_member_info(
        &self,
        qctx: QueryContext,
        msg: QueryMemberInfoMsg,
    ) -> EnterpriseFacadeResult<MemberInfoResponse> {
        let component_contracts = self.component_contracts(qctx.deps)?;
        let user_weight: UserWeightResponse = qctx.deps.querier.query_wasm_smart(
            component_contracts.membership_contract.to_string(),
            &UserWeight(UserWeightParams {
                user: msg.member_address,
            }),
        )?;
        let total_weight: TotalWeightResponse = qctx.deps.querier.query_wasm_smart(
            component_contracts.membership_contract.to_string(),
            &TotalWeight(TotalWeightParams {
                expiration: Never {},
            }),
        )?;

        if total_weight.total_weight.is_zero() {
            Ok(MemberInfoResponse {
                voting_power: Decimal::zero(),
            })
        } else {
            let voting_power =
                Decimal::checked_from_ratio(user_weight.weight, total_weight.total_weight)?;

            Ok(MemberInfoResponse { voting_power })
        }
    }

    fn query_list_multisig_members(
        &self,
        qctx: QueryContext,
        msg: ListMultisigMembersMsg,
    ) -> EnterpriseFacadeResult<MultisigMembersResponse> {
        let dao_info: enterprise_protocol::api::DaoInfoResponse = qctx
            .deps
            .querier
            .query_wasm_smart(self.enterprise_address.to_string(), &DaoInfo {})?;

        match dao_info.dao_type {
            enterprise_protocol::api::DaoType::Multisig => {
                let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

                let members_response: MembersResponse = qctx.deps.querier.query_wasm_smart(
                    membership_contract.to_string(),
                    &Members(MembersParams {
                        start_after: msg.start_after,
                        limit: msg.limit,
                    }),
                )?;

                let members = members_response
                    .members
                    .into_iter()
                    .map(|member| MultisigMember {
                        address: member.user.to_string(),
                        weight: member.weight,
                    })
                    .collect();

                Ok(MultisigMembersResponse { members })
            }
            _ => Err(Dao(UnsupportedOperationForDaoType {
                dao_type: dao_info.dao_type.to_string(),
            })),
        }
    }

    fn query_asset_whitelist(
        &self,
        qctx: QueryContext,
        params: AssetWhitelistParams,
    ) -> EnterpriseFacadeResult<AssetWhitelistResponse> {
        let treasury_address = self
            .component_contracts(qctx.deps)?
            .enterprise_treasury_contract;

        let asset_whitelist: enterprise_treasury_api::api::AssetWhitelistResponse =
            qctx.deps.querier.query_wasm_smart(
                treasury_address.to_string(),
                &AssetWhitelist(enterprise_treasury_api::api::AssetWhitelistParams {
                    start_after: params.start_after,
                    limit: params.limit,
                }),
            )?;

        Ok(AssetWhitelistResponse {
            assets: asset_whitelist.assets,
        })
    }

    fn query_nft_whitelist(
        &self,
        qctx: QueryContext,
        params: NftWhitelistParams,
    ) -> EnterpriseFacadeResult<NftWhitelistResponse> {
        let treasury_address = self
            .component_contracts(qctx.deps)?
            .enterprise_treasury_contract;

        let nft_whitelist: enterprise_treasury_api::api::NftWhitelistResponse =
            qctx.deps.querier.query_wasm_smart(
                treasury_address.to_string(),
                &NftWhitelist(enterprise_treasury_api::api::NftWhitelistParams {
                    start_after: params.start_after,
                    limit: params.limit,
                }),
            )?;

        Ok(NftWhitelistResponse {
            nfts: nft_whitelist.nfts,
        })
    }

    fn query_proposal(
        &self,
        qctx: QueryContext,
        params: ProposalParams,
    ) -> EnterpriseFacadeResult<ProposalResponse> {
        let governance_controller = self
            .component_contracts(qctx.deps)?
            .enterprise_governance_controller_contract;

        let response: enterprise_governance_controller_api::api::ProposalResponse =
            qctx.deps.querier.query_wasm_smart(
                governance_controller.to_string(),
                &enterprise_governance_controller_api::msg::QueryMsg::Proposal(
                    enterprise_governance_controller_api::api::ProposalParams {
                        proposal_id: params.proposal_id,
                    },
                ),
            )?;

        Ok(map_proposal_response(response))
    }

    fn query_proposals(
        &self,
        qctx: QueryContext,
        params: ProposalsParams,
    ) -> EnterpriseFacadeResult<ProposalsResponse> {
        let governance_controller = self
            .component_contracts(qctx.deps)?
            .enterprise_governance_controller_contract;

        let proposals: enterprise_governance_controller_api::api::ProposalsResponse =
            qctx.deps.querier.query_wasm_smart(
                governance_controller.to_string(),
                &enterprise_governance_controller_api::msg::QueryMsg::Proposals(
                    enterprise_governance_controller_api::api::ProposalsParams {
                        filter: params.filter.map(map_proposal_filter),
                        start_after: params.start_after,
                        limit: params.limit,
                    },
                ),
            )?;

        Ok(ProposalsResponse {
            proposals: proposals
                .proposals
                .into_iter()
                .map(map_proposal_response)
                .collect(),
        })
    }

    fn query_proposal_status(
        &self,
        qctx: QueryContext,
        params: ProposalStatusParams,
    ) -> EnterpriseFacadeResult<ProposalStatusResponse> {
        let governance_controller = self
            .component_contracts(qctx.deps)?
            .enterprise_governance_controller_contract;

        let proposal_status: enterprise_governance_controller_api::api::ProposalStatusResponse =
            qctx.deps.querier.query_wasm_smart(
                governance_controller.to_string(),
                &enterprise_governance_controller_api::msg::QueryMsg::ProposalStatus(
                    enterprise_governance_controller_api::api::ProposalStatusParams {
                        proposal_id: params.proposal_id,
                    },
                ),
            )?;

        Ok(ProposalStatusResponse {
            status: map_proposal_status(proposal_status.status),
            expires: proposal_status.expires,
            results: proposal_status.results,
        })
    }

    fn query_member_vote(
        &self,
        qctx: QueryContext,
        params: MemberVoteParams,
    ) -> EnterpriseFacadeResult<MemberVoteResponse> {
        let governance_controller = self
            .component_contracts(qctx.deps)?
            .enterprise_governance_controller_contract;

        let member_vote: enterprise_governance_controller_api::api::MemberVoteResponse =
            qctx.deps.querier.query_wasm_smart(
                governance_controller.to_string(),
                &enterprise_governance_controller_api::msg::QueryMsg::MemberVote(
                    enterprise_governance_controller_api::api::MemberVoteParams {
                        member: params.member,
                        proposal_id: params.proposal_id,
                    },
                ),
            )?;

        Ok(MemberVoteResponse {
            vote: member_vote.vote,
        })
    }

    fn query_proposal_votes(
        &self,
        qctx: QueryContext,
        params: ProposalVotesParams,
    ) -> EnterpriseFacadeResult<ProposalVotesResponse> {
        let governance_controller = self
            .component_contracts(qctx.deps)?
            .enterprise_governance_controller_contract;

        let proposal_votes: enterprise_governance_controller_api::api::ProposalVotesResponse =
            qctx.deps.querier.query_wasm_smart(
                governance_controller.to_string(),
                &enterprise_governance_controller_api::msg::QueryMsg::ProposalVotes(
                    enterprise_governance_controller_api::api::ProposalVotesParams {
                        proposal_id: params.proposal_id,
                        start_after: params.start_after,
                        limit: params.limit,
                    },
                ),
            )?;

        Ok(ProposalVotesResponse {
            votes: proposal_votes.votes,
        })
    }

    fn query_user_stake(
        &self,
        qctx: QueryContext,
        params: UserStakeParams,
    ) -> EnterpriseFacadeResult<UserStakeResponse> {
        let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

        match self.get_dao_type(qctx.deps)? {
            DaoType::Denom => {
                let denom_stake: UserWeightResponse = qctx.deps.querier.query_wasm_smart(
                    membership_contract.to_string(),
                    &UserWeight(UserWeightParams { user: params.user }),
                )?;

                Ok(UserStakeResponse {
                    user_stake: UserStake::Denom(DenomUserStake {
                        amount: denom_stake.weight,
                    }),
                })
            }
            DaoType::Token => {
                let token_stake: UserWeightResponse = qctx.deps.querier.query_wasm_smart(
                    membership_contract.to_string(),
                    &UserWeight(UserWeightParams { user: params.user }),
                )?;

                Ok(UserStakeResponse {
                    user_stake: UserStake::Token(TokenUserStake {
                        amount: token_stake.weight,
                    }),
                })
            }
            DaoType::Nft => {
                let nft_stake: UserNftStakeResponse = qctx.deps.querier.query_wasm_smart(
                    membership_contract.to_string(),
                    &nft_staking_api::msg::QueryMsg::UserStake(UserNftStakeParams {
                        user: params.user,
                        start_after: params.start_after,
                        limit: params.limit,
                    }),
                )?;

                Ok(UserStakeResponse {
                    user_stake: UserStake::Nft(NftUserStake {
                        tokens: nft_stake.tokens,
                        amount: nft_stake.total_user_stake,
                    }),
                })
            }
            DaoType::Multisig => Ok(UserStakeResponse {
                user_stake: UserStake::None,
            }),
        }
    }

    fn query_total_staked_amount(
        &self,
        qctx: QueryContext,
    ) -> EnterpriseFacadeResult<TotalStakedAmountResponse> {
        match self.get_dao_type(qctx.deps)? {
            DaoType::Denom | DaoType::Token | DaoType::Nft => {
                let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

                let total_weight: TotalWeightResponse = qctx.deps.querier.query_wasm_smart(
                    membership_contract.to_string(),
                    &TotalWeight(TotalWeightParams {
                        expiration: Never {},
                    }),
                )?;

                Ok(TotalStakedAmountResponse {
                    total_staked_amount: total_weight.total_weight,
                })
            }
            DaoType::Multisig => Ok(TotalStakedAmountResponse {
                total_staked_amount: Uint128::zero(),
            }),
        }
    }

    fn query_staked_nfts(
        &self,
        qctx: QueryContext,
        params: StakedNftsParams,
    ) -> EnterpriseFacadeResult<StakedNftsResponse> {
        let dao_type = self.get_dao_type(qctx.deps)?;

        match dao_type {
            DaoType::Nft => {
                let nft_membership_contract =
                    self.component_contracts(qctx.deps)?.membership_contract;

                let staked_nfts_response: nft_staking_api::api::StakedNftsResponse =
                    qctx.deps.querier.query_wasm_smart(
                        nft_membership_contract.to_string(),
                        &StakedNfts(nft_staking_api::api::StakedNftsParams {
                            start_after: params.start_after,
                            limit: params.limit,
                        }),
                    )?;

                Ok(StakedNftsResponse {
                    nfts: staked_nfts_response.nfts,
                })
            }
            DaoType::Denom | DaoType::Token | DaoType::Multisig => {
                Ok(StakedNftsResponse { nfts: vec![] })
            }
        }
    }

    fn query_claims(
        &self,
        qctx: QueryContext,
        params: ClaimsParams,
    ) -> EnterpriseFacadeResult<ClaimsResponse> {
        let dao_type = self.get_dao_type(qctx.deps)?;

        match dao_type {
            DaoType::Denom => {
                let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

                let response: denom_staking_api::api::ClaimsResponse =
                    qctx.deps.querier.query_wasm_smart(
                        membership_contract.to_string(),
                        &denom_staking_api::msg::QueryMsg::Claims(
                            denom_staking_api::api::ClaimsParams { user: params.owner },
                        ),
                    )?;

                Ok(map_denom_claims_response(response))
            }
            DaoType::Token => {
                let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

                let response: token_staking_api::api::ClaimsResponse =
                    qctx.deps.querier.query_wasm_smart(
                        membership_contract.to_string(),
                        &token_staking_api::msg::QueryMsg::Claims(
                            token_staking_api::api::ClaimsParams { user: params.owner },
                        ),
                    )?;

                Ok(map_token_claims_response(response))
            }
            DaoType::Nft => {
                let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

                let response: nft_staking_api::api::ClaimsResponse =
                    qctx.deps.querier.query_wasm_smart(
                        membership_contract.to_string(),
                        &nft_staking_api::msg::QueryMsg::Claims(
                            nft_staking_api::api::ClaimsParams { user: params.owner },
                        ),
                    )?;

                Ok(map_nft_claims_response(response))
            }
            DaoType::Multisig => Ok(ClaimsResponse { claims: vec![] }),
        }
    }

    fn query_releasable_claims(
        &self,
        qctx: QueryContext,
        params: ClaimsParams,
    ) -> EnterpriseFacadeResult<ClaimsResponse> {
        let dao_type = self.get_dao_type(qctx.deps)?;

        match dao_type {
            DaoType::Denom => {
                let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

                let response: denom_staking_api::api::ClaimsResponse =
                    qctx.deps.querier.query_wasm_smart(
                        membership_contract.to_string(),
                        &denom_staking_api::msg::QueryMsg::ReleasableClaims(
                            denom_staking_api::api::ClaimsParams { user: params.owner },
                        ),
                    )?;

                Ok(map_denom_claims_response(response))
            }
            DaoType::Token => {
                let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

                let response: token_staking_api::api::ClaimsResponse =
                    qctx.deps.querier.query_wasm_smart(
                        membership_contract.to_string(),
                        &token_staking_api::msg::QueryMsg::ReleasableClaims(
                            token_staking_api::api::ClaimsParams { user: params.owner },
                        ),
                    )?;

                Ok(map_token_claims_response(response))
            }
            DaoType::Nft => {
                let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

                let response: nft_staking_api::api::ClaimsResponse =
                    qctx.deps.querier.query_wasm_smart(
                        membership_contract.to_string(),
                        &nft_staking_api::msg::QueryMsg::ReleasableClaims(
                            nft_staking_api::api::ClaimsParams { user: params.owner },
                        ),
                    )?;

                Ok(map_nft_claims_response(response))
            }
            DaoType::Multisig => Ok(ClaimsResponse { claims: vec![] }),
        }
    }

    fn query_cross_chain_treasuries(
        &self,
        qctx: QueryContext,
        params: CrossChainTreasuriesParams,
    ) -> EnterpriseFacadeResult<CrossChainTreasuriesResponse> {
        Ok(qctx.deps.querier.query_wasm_smart(
            self.component_contracts(qctx.deps)?
                .enterprise_outposts_contract
                .to_string(),
            &CrossChainTreasuries(params),
        )?)
    }

    fn query_has_incomplete_v2_migration(
        &self,
        qctx: QueryContext,
    ) -> EnterpriseFacadeResult<HasIncompleteV2MigrationResponse> {
        let component_contracts = self.component_contracts(qctx.deps)?;

        let response: HasIncompleteV2MigrationResponse = qctx.deps.querier.query_wasm_smart(
            component_contracts.enterprise_treasury_contract,
            &HasIncompleteV2Migration {},
        )?;

        Ok(response)
    }

    fn adapt_create_proposal(
        &self,
        qctx: QueryContext,
        params: CreateProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        let governance_controller = self
            .component_contracts(qctx.deps)?
            .enterprise_governance_controller_contract;

        Ok(adapter_response_single_execute_msg(
            governance_controller,
            serde_json_wasm::to_string(&CreateProposal(
                enterprise_governance_controller_api::api::CreateProposalMsg {
                    title: params.title,
                    description: params.description,
                    proposal_actions: params.proposal_actions,
                },
            ))?,
            vec![],
        ))
    }

    fn adapt_create_proposal_with_denom_deposit(
        &self,
        qctx: QueryContext,
        params: CreateProposalWithDenomDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        let dao_type = self.get_dao_type(qctx.deps)?;

        if dao_type != DaoType::Denom {
            return Err(Dao(UnsupportedOperationForDaoType {
                dao_type: dao_type.to_string(),
            }));
        }

        let governance_controller = self
            .component_contracts(qctx.deps)?
            .enterprise_governance_controller_contract;
        let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

        let denom_config: DenomConfigResponse = qctx
            .deps
            .querier
            .query_wasm_smart(membership_contract.to_string(), &DenomConfig {})?;

        let create_proposal_with_denom_deposit_msg = AdaptedMsg::Execute(AdaptedExecuteMsg {
            target_contract: governance_controller,
            msg: serde_json_wasm::to_string(&CreateProposal(
                enterprise_governance_controller_api::api::CreateProposalMsg {
                    title: params.create_proposal_msg.title,
                    description: params.create_proposal_msg.description,
                    proposal_actions: params.create_proposal_msg.proposal_actions,
                },
            ))?,
            funds: coins(params.deposit_amount.u128(), denom_config.denom),
        });

        Ok(AdapterResponse {
            msgs: vec![create_proposal_with_denom_deposit_msg],
        })
    }

    fn adapt_create_proposal_with_token_deposit(
        &self,
        qctx: QueryContext,
        params: CreateProposalWithTokenDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        let dao_type = self.get_dao_type(qctx.deps)?;

        match dao_type {
            DaoType::Token => {
                let governance_controller = self
                    .component_contracts(qctx.deps)?
                    .enterprise_governance_controller_contract;
                let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

                let token_config: TokenConfigResponse = qctx
                    .deps
                    .querier
                    .query_wasm_smart(membership_contract.to_string(), &TokenConfig {})?;

                Ok(adapter_response_single_execute_msg(
                    token_config.token_contract,
                    serde_json_wasm::to_string(&cw20::Cw20ExecuteMsg::Send {
                        contract: governance_controller.to_string(),
                        amount: params.deposit_amount,
                        msg: to_json_binary(
                            &enterprise_governance_controller_api::msg::Cw20HookMsg::CreateProposal(
                                enterprise_governance_controller_api::api::CreateProposalMsg {
                                    title: params.create_proposal_msg.title,
                                    description: params.create_proposal_msg.description,
                                    proposal_actions: params.create_proposal_msg.proposal_actions,
                                },
                            ),
                        )?,
                    })?,
                    vec![],
                ))
            }
            _ => Err(Dao(UnsupportedOperationForDaoType {
                dao_type: dao_type.to_string(),
            })),
        }
    }

    fn adapt_create_proposal_with_nft_deposit(
        &self,
        qctx: QueryContext,
        params: CreateProposalWithNftDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        let dao_type = self.get_dao_type(qctx.deps)?;

        if dao_type != DaoType::Nft {
            return Err(Dao(UnsupportedOperationForDaoType {
                dao_type: dao_type.to_string(),
            }));
        }

        let governance_controller = self
            .component_contracts(qctx.deps)?
            .enterprise_governance_controller_contract;
        let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

        let nft_config: NftConfigResponse = qctx
            .deps
            .querier
            .query_wasm_smart(membership_contract.to_string(), &NftConfig {})?;

        // give governance controller allowance over deposit tokens
        let allow_deposit_tokens_for_governance_controller = params
            .deposit_tokens
            .iter()
            .map(|token_id| {
                serde_json_wasm::to_string(&Approve {
                    spender: governance_controller.to_string(),
                    token_id: token_id.to_string(),
                    expires: None,
                })
            })
            .map(|msg_json_res| {
                msg_json_res.map(|msg_json| {
                    AdaptedMsg::Execute(AdaptedExecuteMsg {
                        target_contract: nft_config.nft_contract.clone(),
                        msg: msg_json,
                        funds: vec![],
                    })
                })
            })
            .collect::<serde_json_wasm::ser::Result<Vec<AdaptedMsg>>>()?;

        let mut msgs = allow_deposit_tokens_for_governance_controller;

        let create_proposal_with_nft_deposit_msg = AdaptedMsg::Execute(AdaptedExecuteMsg {
            target_contract: governance_controller,
            msg: serde_json_wasm::to_string(&CreateProposalWithNftDeposit(params))?,
            funds: vec![],
        });

        msgs.push(create_proposal_with_nft_deposit_msg);

        Ok(AdapterResponse { msgs })
    }

    fn adapt_create_council_proposal(
        &self,
        qctx: QueryContext,
        params: CreateProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        let governance_controller = self
            .component_contracts(qctx.deps)?
            .enterprise_governance_controller_contract;

        Ok(adapter_response_single_execute_msg(
            governance_controller,
            serde_json_wasm::to_string(
                &enterprise_governance_controller_api::msg::ExecuteMsg::CreateCouncilProposal(
                    enterprise_governance_controller_api::api::CreateProposalMsg {
                        title: params.title,
                        description: params.description,
                        proposal_actions: params.proposal_actions,
                    },
                ),
            )?,
            vec![],
        ))
    }

    fn adapt_cast_vote(
        &self,
        qctx: QueryContext,
        params: CastVoteMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        let governance_controller = self
            .component_contracts(qctx.deps)?
            .enterprise_governance_controller_contract;

        Ok(adapter_response_single_execute_msg(
            governance_controller,
            serde_json_wasm::to_string(
                &enterprise_governance_controller_api::msg::ExecuteMsg::CastVote(
                    enterprise_governance_controller_api::api::CastVoteMsg {
                        proposal_id: params.proposal_id,
                        outcome: params.outcome,
                    },
                ),
            )?,
            vec![],
        ))
    }

    fn adapt_cast_council_vote(
        &self,
        qctx: QueryContext,
        params: CastVoteMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        let governance_controller = self
            .component_contracts(qctx.deps)?
            .enterprise_governance_controller_contract;

        Ok(adapter_response_single_execute_msg(
            governance_controller,
            serde_json_wasm::to_string(
                &enterprise_governance_controller_api::msg::ExecuteMsg::CastCouncilVote(
                    enterprise_governance_controller_api::api::CastVoteMsg {
                        proposal_id: params.proposal_id,
                        outcome: params.outcome,
                    },
                ),
            )?,
            vec![],
        ))
    }

    fn adapt_execute_proposal(
        &self,
        qctx: QueryContext,
        params: ExecuteProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        let proposal_response = self.query_proposal(
            qctx.clone(),
            ProposalParams {
                proposal_id: params.proposal_id,
            },
        )?;

        let treasury_cross_chain_msgs_count = proposal_response
            .proposal
            .proposal_actions
            .into_iter()
            .filter(|action| match action {
                ProposalAction::UpdateAssetWhitelist(msg) => msg.remote_treasury_target.is_some(),
                ProposalAction::UpdateNftWhitelist(msg) => msg.remote_treasury_target.is_some(),
                ProposalAction::RequestFundingFromDao(msg) => msg.remote_treasury_target.is_some(),
                ProposalAction::ExecuteTreasuryMsgs(msg) => msg.remote_treasury_target.is_some(),
                ProposalAction::DeployCrossChainTreasury(_) => true,
                _ => false,
            })
            .count() as u128;

        let component_contracts = self.component_contracts(qctx.deps)?;

        let execute_proposal_submsg = AdaptedMsg::Execute(AdaptedExecuteMsg {
            target_contract: component_contracts.enterprise_governance_controller_contract,
            msg: serde_json_wasm::to_string(&ExecuteProposal(
                enterprise_governance_controller_api::api::ExecuteProposalMsg {
                    proposal_id: params.proposal_id,
                },
            ))?,
            funds: vec![],
        });

        let msgs = if treasury_cross_chain_msgs_count != 0u128 {
            let fund_outposts_contract_submsg = AdaptedMsg::Bank(AdaptedBankMsg {
                receiver: component_contracts.enterprise_outposts_contract,
                funds: coins(treasury_cross_chain_msgs_count, "uluna"),
            });
            vec![fund_outposts_contract_submsg, execute_proposal_submsg]
        } else {
            vec![execute_proposal_submsg]
        };

        Ok(AdapterResponse { msgs })
    }

    fn adapt_stake(
        &self,
        qctx: QueryContext,
        params: StakeMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

        match params {
            StakeMsg::Cw20(msg) => {
                let token_config: TokenConfigResponse = qctx
                    .deps
                    .querier
                    .query_wasm_smart(membership_contract.to_string(), &TokenConfig {})?;
                let msg = cw20::Cw20ExecuteMsg::Send {
                    contract: membership_contract.to_string(),
                    amount: msg.amount,
                    msg: to_json_binary(&token_staking_api::msg::Cw20HookMsg::Stake {
                        user: msg.user,
                    })?,
                };
                Ok(adapter_response_single_execute_msg(
                    token_config.token_contract,
                    serde_json_wasm::to_string(&msg)?,
                    vec![],
                ))
            }
            StakeMsg::Cw721(msg) => {
                let nft_config: NftConfigResponse = qctx
                    .deps
                    .querier
                    .query_wasm_smart(membership_contract.to_string(), &NftConfig {})?;

                let stake_msg_binary =
                    to_json_binary(&nft_staking_api::msg::Cw721HookMsg::Stake {
                        user: msg.user.clone(),
                    })?;

                let msgs = msg
                    .tokens
                    .into_iter()
                    .map(|token_id| cw721::Cw721ExecuteMsg::SendNft {
                        contract: membership_contract.to_string(),
                        token_id,
                        msg: stake_msg_binary.clone(),
                    })
                    .map(|send_nft_msg| {
                        serde_json_wasm::to_string(&send_nft_msg).map(|send_nft_msg_json| {
                            AdaptedMsg::Execute(AdaptedExecuteMsg {
                                target_contract: nft_config.nft_contract.clone(),
                                msg: send_nft_msg_json,
                                funds: vec![],
                            })
                        })
                    })
                    .collect::<serde_json_wasm::ser::Result<Vec<AdaptedMsg>>>()?;
                Ok(AdapterResponse { msgs })
            }
            StakeMsg::Denom(msg) => {
                let denom_config: DenomConfigResponse = qctx
                    .deps
                    .querier
                    .query_wasm_smart(membership_contract.to_string(), &DenomConfig {})?;
                Ok(adapter_response_single_execute_msg(
                    membership_contract,
                    serde_json_wasm::to_string(&denom_staking_api::msg::ExecuteMsg::Stake {})?,
                    coins(msg.amount.u128(), denom_config.denom),
                ))
            }
        }
    }

    fn adapt_unstake(
        &self,
        qctx: QueryContext,
        params: UnstakeMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

        match params {
            UnstakeMsg::Cw20(msg) => Ok(adapter_response_single_execute_msg(
                membership_contract,
                serde_json_wasm::to_string(&token_staking_api::msg::ExecuteMsg::Unstake(
                    token_staking_api::api::UnstakeMsg { amount: msg.amount },
                ))?,
                vec![],
            )),
            UnstakeMsg::Cw721(msg) => Ok(adapter_response_single_execute_msg(
                membership_contract,
                serde_json_wasm::to_string(&nft_staking_api::msg::ExecuteMsg::Unstake(
                    nft_staking_api::api::UnstakeMsg {
                        nft_ids: msg.tokens,
                    },
                ))?,
                vec![],
            )),
            UnstakeMsg::Denom(msg) => Ok(adapter_response_single_execute_msg(
                membership_contract,
                serde_json_wasm::to_string(&denom_staking_api::msg::ExecuteMsg::Unstake(
                    denom_staking_api::api::UnstakeMsg { amount: msg.amount },
                ))?,
                vec![],
            )),
        }
    }

    fn adapt_claim(&self, qctx: QueryContext) -> EnterpriseFacadeResult<AdapterResponse> {
        let dao_type = self.get_dao_type(qctx.deps)?;

        match dao_type {
            DaoType::Denom => {
                let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

                Ok(adapter_response_single_execute_msg(
                    membership_contract,
                    serde_json_wasm::to_string(&denom_staking_api::msg::ExecuteMsg::Claim(
                        denom_staking_api::api::ClaimMsg { user: None },
                    ))?,
                    vec![],
                ))
            }
            DaoType::Token => {
                let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

                Ok(adapter_response_single_execute_msg(
                    membership_contract,
                    serde_json_wasm::to_string(&token_staking_api::msg::ExecuteMsg::Claim(
                        token_staking_api::api::ClaimMsg { user: None },
                    ))?,
                    vec![],
                ))
            }
            DaoType::Nft => {
                let membership_contract = self.component_contracts(qctx.deps)?.membership_contract;

                Ok(adapter_response_single_execute_msg(
                    membership_contract,
                    serde_json_wasm::to_string(&nft_staking_api::msg::ExecuteMsg::Claim(
                        nft_staking_api::api::ClaimMsg { user: None },
                    ))?,
                    vec![],
                ))
            }
            DaoType::Multisig => Err(Dao(UnsupportedOperationForDaoType {
                dao_type: dao_type.to_string(),
            })),
        }
    }
}

impl EnterpriseFacadeV2 {
    fn component_contracts(
        &self,
        deps: Deps,
    ) -> EnterpriseFacadeResult<ComponentContractsResponse> {
        let component_contracts = deps
            .querier
            .query_wasm_smart(self.enterprise_address.to_string(), &ComponentContracts {})?;

        Ok(component_contracts)
    }

    fn get_dao_type(&self, deps: Deps) -> EnterpriseFacadeResult<DaoType> {
        let dao_info: enterprise_protocol::api::DaoInfoResponse = deps
            .querier
            .query_wasm_smart(self.enterprise_address.to_string(), &DaoInfo {})?;

        Ok(map_dao_type(dao_info.dao_type))
    }
}

fn map_denom_claims_response(response: denom_staking_api::api::ClaimsResponse) -> ClaimsResponse {
    ClaimsResponse {
        claims: response
            .claims
            .into_iter()
            .map(|claim| Claim {
                asset: ClaimAsset::Denom(DenomClaimAsset {
                    amount: claim.amount,
                }),
                release_at: claim.release_at,
            })
            .collect(),
    }
}

fn map_token_claims_response(response: token_staking_api::api::ClaimsResponse) -> ClaimsResponse {
    ClaimsResponse {
        claims: response
            .claims
            .into_iter()
            .map(|claim| Claim {
                asset: ClaimAsset::Cw20(Cw20ClaimAsset {
                    amount: claim.amount,
                }),
                release_at: claim.release_at,
            })
            .collect(),
    }
}

fn map_nft_claims_response(response: nft_staking_api::api::ClaimsResponse) -> ClaimsResponse {
    ClaimsResponse {
        claims: response
            .claims
            .into_iter()
            .map(|claim| Claim {
                asset: ClaimAsset::Cw721(Cw721ClaimAsset {
                    tokens: claim.nft_ids,
                }),
                release_at: claim.release_at,
            })
            .collect(),
    }
}

fn map_dao_type(dao_type: enterprise_protocol::api::DaoType) -> DaoType {
    match dao_type {
        enterprise_protocol::api::DaoType::Denom => DaoType::Denom,
        enterprise_protocol::api::DaoType::Token => DaoType::Token,
        enterprise_protocol::api::DaoType::Nft => DaoType::Nft,
        enterprise_protocol::api::DaoType::Multisig => DaoType::Multisig,
    }
}

fn map_proposal_status(
    status: enterprise_governance_controller_api::api::ProposalStatus,
) -> ProposalStatus {
    match status {
        enterprise_governance_controller_api::api::ProposalStatus::InProgress => {
            ProposalStatus::InProgress
        }
        enterprise_governance_controller_api::api::ProposalStatus::InProgressCanExecuteEarly => {
            ProposalStatus::InProgressCanExecuteEarly
        }
        enterprise_governance_controller_api::api::ProposalStatus::Passed => ProposalStatus::Passed,
        enterprise_governance_controller_api::api::ProposalStatus::Rejected => {
            ProposalStatus::Rejected
        }
        enterprise_governance_controller_api::api::ProposalStatus::Executed => {
            ProposalStatus::Executed
        }
    }
}

fn map_proposal_filter(
    filter: ProposalStatusFilter,
) -> enterprise_governance_controller_api::api::ProposalStatusFilter {
    match filter {
        ProposalStatusFilter::InProgress => {
            enterprise_governance_controller_api::api::ProposalStatusFilter::InProgress
        }
        ProposalStatusFilter::Passed => {
            enterprise_governance_controller_api::api::ProposalStatusFilter::Passed
        }
        ProposalStatusFilter::Rejected => {
            enterprise_governance_controller_api::api::ProposalStatusFilter::Rejected
        }
    }
}

fn map_proposal(proposal: enterprise_governance_controller_api::api::Proposal) -> Proposal {
    let proposal_type = match proposal.proposal_type {
        enterprise_governance_controller_api::api::ProposalType::General => ProposalType::General,
        enterprise_governance_controller_api::api::ProposalType::Council => ProposalType::Council,
    };
    Proposal {
        proposal_type,
        id: proposal.id,
        proposer: proposal.proposer,
        title: proposal.title,
        description: proposal.description,
        status: map_proposal_status(proposal.status),
        started_at: proposal.started_at,
        expires: proposal.expires,
        proposal_actions: proposal.proposal_actions,
    }
}

fn map_proposal_response(
    response: enterprise_governance_controller_api::api::ProposalResponse,
) -> ProposalResponse {
    ProposalResponse {
        proposal: map_proposal(response.proposal),
        proposal_status: map_proposal_status(response.proposal_status),
        results: response.results,
        total_votes_available: response.total_votes_available,
    }
}
