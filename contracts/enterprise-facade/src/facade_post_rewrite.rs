use crate::facade::EnterpriseFacade;
use common::cw::{Context, QueryContext};
use cosmwasm_std::{wasm_execute, Addr, Decimal, Deps, Response, SubMsg, Uint128, Uint64};
use cw_utils::Expiration::Never;
use enterprise_facade_api::api::{
    AssetWhitelistParams, AssetWhitelistResponse, Claim, ClaimAsset, ClaimsParams, ClaimsResponse,
    Cw20ClaimAsset, Cw721ClaimAsset, DaoCouncil, DaoGovConfig, DaoInfoResponse, DaoMetadata,
    DaoSocialData, DaoType, ExecuteProposalMsg, ListMultisigMembersMsg, Logo, MemberInfoResponse,
    MemberVoteParams, MemberVoteResponse, MultisigMember, MultisigMembersResponse, NftUserStake,
    NftWhitelistParams, NftWhitelistResponse, Proposal, ProposalActionType, ProposalParams,
    ProposalResponse, ProposalStatus, ProposalStatusFilter, ProposalStatusParams,
    ProposalStatusResponse, ProposalType, ProposalVotesParams, ProposalVotesResponse,
    ProposalsParams, ProposalsResponse, QueryMemberInfoMsg, StakedNftsParams, StakedNftsResponse,
    TokenUserStake, TotalStakedAmountResponse, UserStake, UserStakeParams, UserStakeResponse,
};
use enterprise_facade_api::error::DaoError::UnsupportedOperationForDaoType;
use enterprise_facade_api::error::EnterpriseFacadeError::Dao;
use enterprise_facade_api::error::EnterpriseFacadeResult;
use enterprise_governance_controller_api::api::GovConfigResponse;
use enterprise_governance_controller_api::msg::ExecuteMsg::ExecuteProposal;
use enterprise_protocol::api::ComponentContractsResponse;
use enterprise_protocol::msg::QueryMsg::{ComponentContracts, DaoInfo};
use enterprise_treasury_api::msg::QueryMsg::{AssetWhitelist, NftWhitelist};
use membership_common_api::api::{
    MembersParams, MembersResponse, TotalWeightParams, TotalWeightResponse, UserWeightParams,
    UserWeightResponse,
};
use membership_common_api::msg::QueryMsg::{TotalWeight, UserWeight};
use nft_staking_api::api::{NftConfigResponse, UserNftStakeParams, UserNftStakeResponse};
use nft_staking_api::msg::QueryMsg::NftConfig;
use token_staking_api::api::TokenConfigResponse;
use token_staking_api::msg::QueryMsg::TokenConfig;

/// Facade implementation for v1.0.0 of Enterprise (post-contract-rewrite).
pub struct EnterpriseFacadePostRewrite {
    pub enterprise_treasury_address: Addr,
    pub enterprise_address: Addr,
}

impl EnterpriseFacade for EnterpriseFacadePostRewrite {
    fn execute_proposal(
        &self,
        ctx: &mut Context,
        msg: ExecuteProposalMsg,
    ) -> EnterpriseFacadeResult<Response> {
        let component_contracts = self.component_contracts(ctx.deps.as_ref())?;

        let governance_controller_contract =
            component_contracts.enterprise_governance_controller_contract;
        let submsg = SubMsg::new(wasm_execute(
            governance_controller_contract.to_string(),
            &ExecuteProposal(
                enterprise_governance_controller_api::api::ExecuteProposalMsg {
                    proposal_id: msg.proposal_id,
                },
            ),
            vec![],
        )?);

        Ok(Response::new().add_submessage(submsg))
    }

    fn query_dao_info(&self, qctx: QueryContext) -> EnterpriseFacadeResult<DaoInfoResponse> {
        let dao_info: enterprise_protocol::api::DaoInfoResponse = qctx
            .deps
            .querier
            .query_wasm_smart(self.enterprise_address.to_string(), &DaoInfo {})?;

        let logo = match dao_info.metadata.logo {
            enterprise_protocol::api::Logo::Url(url) => Logo::Url(url),
            enterprise_protocol::api::Logo::None => Logo::None,
        };

        let dao_type = map_dao_type(dao_info.dao_type);

        let component_contracts = self.component_contracts(qctx.deps)?;

        let gov_config: GovConfigResponse = qctx.deps.querier.query_wasm_smart(
            component_contracts
                .enterprise_governance_controller_contract
                .to_string(),
            &enterprise_governance_controller_api::msg::QueryMsg::GovConfig {},
        )?;

        // get the membership contract, which actually used to mean the CW20 / CW721 contract, not the new membership contracts
        let dao_membership_contract = match dao_type {
            DaoType::Token => {
                let token_config: TokenConfigResponse = qctx.deps.querier.query_wasm_smart(
                    gov_config.dao_membership_contract.to_string(),
                    &TokenConfig {},
                )?;
                token_config.token_contract
            }
            DaoType::Nft => {
                let nft_config: NftConfigResponse = qctx.deps.querier.query_wasm_smart(
                    gov_config.dao_membership_contract.to_string(),
                    &NftConfig {},
                )?;
                nft_config.nft_contract
            }
            DaoType::Multisig => {
                // doesn't make too much sense, but kept for backwards-compatibility since this was the previous behavior
                self.enterprise_address.clone()
            }
        };

        let dao_council = gov_config.council_gov_config.map(|config| {
            let allowed_proposal_action_types = config
                .allowed_proposal_action_types
                .into_iter()
                .map(map_proposal_action_type)
                .collect();

            DaoCouncil {
                members: vec![], // TODO: query from council membership contract
                allowed_proposal_action_types,
                quorum: config.quorum,
                threshold: config.threshold,
            }
        });

        // Map DAO version to version code, formula: 100*100*(major) + 100*(minor) + (patch)
        let version = dao_info.dao_version;
        let major_component = Uint64::from(version.major).checked_mul(10_000u64.into())?;
        let minor_component = Uint64::from(version.minor).checked_mul(100u64.into())?;
        let patch_component = Uint64::from(version.patch);
        let dao_code_version = major_component
            .checked_add(minor_component)?
            .checked_add(patch_component)?;

        Ok(DaoInfoResponse {
            creation_date: dao_info.creation_date,
            metadata: DaoMetadata {
                name: dao_info.metadata.name,
                description: dao_info.metadata.description,
                logo,
                socials: DaoSocialData {
                    github_username: dao_info.metadata.socials.github_username,
                    discord_username: dao_info.metadata.socials.discord_username,
                    twitter_username: dao_info.metadata.socials.twitter_username,
                    telegram_username: dao_info.metadata.socials.telegram_username,
                },
            },
            gov_config: DaoGovConfig {
                quorum: gov_config.gov_config.quorum,
                threshold: gov_config.gov_config.threshold,
                veto_threshold: gov_config.gov_config.veto_threshold,
                vote_duration: gov_config.gov_config.vote_duration,
                unlocking_period: gov_config.gov_config.unlocking_period,
                minimum_deposit: gov_config.gov_config.minimum_deposit,
                allow_early_proposal_execution: gov_config
                    .gov_config
                    .allow_early_proposal_execution,
            },
            dao_council,
            dao_type,
            dao_membership_contract,
            enterprise_factory_contract: component_contracts.enterprise_factory_contract,
            funds_distributor_contract: component_contracts.funds_distributor_contract,
            dao_code_version,
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
                    &membership_common_api::msg::QueryMsg::Members(MembersParams {
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
        let asset_whitelist: enterprise_treasury_api::api::AssetWhitelistResponse =
            qctx.deps.querier.query_wasm_smart(
                self.enterprise_treasury_address.to_string(),
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
        let nft_whitelist: enterprise_treasury_api::api::NftWhitelistResponse =
            qctx.deps.querier.query_wasm_smart(
                self.enterprise_treasury_address.to_string(),
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
                    &UserNftStakeParams {
                        user: params.user,
                        start_after: params.start_after,
                        limit: params.limit,
                    },
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
            DaoType::Token | DaoType::Nft => {
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
        _params: StakedNftsParams,
    ) -> EnterpriseFacadeResult<StakedNftsResponse> {
        let dao_type = self.get_dao_type(qctx.deps)?;

        match dao_type {
            DaoType::Nft => {
                // TODO: add a query for this to the NFT staking memberhsip contract
                let _nft_membership_contract =
                    self.component_contracts(qctx.deps)?.membership_contract;

                Ok(StakedNftsResponse { nfts: vec![] })
            }
            DaoType::Token | DaoType::Multisig => Ok(StakedNftsResponse { nfts: vec![] }),
        }
    }

    fn query_claims(
        &self,
        qctx: QueryContext,
        params: ClaimsParams,
    ) -> EnterpriseFacadeResult<ClaimsResponse> {
        let dao_type = self.get_dao_type(qctx.deps)?;

        match dao_type {
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
}

impl EnterpriseFacadePostRewrite {
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
        enterprise_protocol::api::DaoType::Token => DaoType::Token,
        enterprise_protocol::api::DaoType::Nft => DaoType::Nft,
        enterprise_protocol::api::DaoType::Multisig => DaoType::Multisig,
    }
}

fn map_proposal_action_type(
    action_type: enterprise_governance_controller_api::api::ProposalActionType,
) -> ProposalActionType {
    match action_type {
        enterprise_governance_controller_api::api::ProposalActionType::UpdateMetadata => ProposalActionType::UpdateMetadata,
        enterprise_governance_controller_api::api::ProposalActionType::UpdateGovConfig => ProposalActionType::UpdateGovConfig,
        enterprise_governance_controller_api::api::ProposalActionType::UpdateCouncil => ProposalActionType::UpdateCouncil,
        enterprise_governance_controller_api::api::ProposalActionType::UpdateAssetWhitelist => ProposalActionType::UpdateAssetWhitelist,
        enterprise_governance_controller_api::api::ProposalActionType::UpdateNftWhitelist => ProposalActionType::UpdateNftWhitelist,
        enterprise_governance_controller_api::api::ProposalActionType::RequestFundingFromDao => ProposalActionType::RequestFundingFromDao,
        enterprise_governance_controller_api::api::ProposalActionType::UpgradeDao => ProposalActionType::UpgradeDao,
        enterprise_governance_controller_api::api::ProposalActionType::ExecuteMsgs => ProposalActionType::ExecuteMsgs,
        enterprise_governance_controller_api::api::ProposalActionType::ModifyMultisigMembership => ProposalActionType::ModifyMultisigMembership,
        enterprise_governance_controller_api::api::ProposalActionType::DistributeFunds => ProposalActionType::DistributeFunds,
        enterprise_governance_controller_api::api::ProposalActionType::UpdateMinimumWeightForRewards => ProposalActionType::UpdateMinimumWeightForRewards,
    }
}

fn map_proposal_status(
    status: enterprise_governance_controller_api::api::ProposalStatus,
) -> ProposalStatus {
    match status {
        enterprise_governance_controller_api::api::ProposalStatus::InProgress => {
            ProposalStatus::InProgress
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
