use crate::state::ENTERPRISE_VERSIONING;
use crate::v1_structs;
use crate::v1_structs::ExecuteV1Msg::{
    CastCouncilVote, CastVote, Claim, CreateCouncilProposal, CreateProposal, Unstake,
};
use crate::v1_structs::ProposalActionV1::{
    DistributeFunds, ExecuteMsgs, ModifyMultisigMembership, RequestFundingFromDao,
    UpdateAssetWhitelist, UpdateCouncil, UpdateGovConfig, UpdateMetadata,
    UpdateMinimumWeightForRewards, UpdateNftWhitelist, UpgradeDao,
};
use crate::v1_structs::QueryV1Msg::{
    AssetWhitelist, Claims, DaoInfo, ListMultisigMembers, MemberInfo, MemberVote, NftWhitelist,
    ProposalVotes, Proposals, ReleasableClaims, StakedNfts, TotalStakedAmount, UserStake,
};
use crate::v1_structs::{
    CreateProposalV1Msg, Cw20HookV1Msg, Cw721HookV1Msg, DaoInfoResponseV1, ExecuteV1Msg,
    ProposalActionV1, ProposalResponseV1, ProposalsResponseV1, TreasuryV1_0_0MigrationMsg,
    UnstakeCw20V1Msg, UnstakeCw721V1Msg, UnstakeV1Msg, UpgradeDaoV1Msg, UserStakeV1Params,
};
use common::cw::QueryContext;
use cosmwasm_schema::serde::de::DeserializeOwned;
use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{to_json_binary, Addr, Deps, Empty, StdError, StdResult};
use cw_utils::Expiration;
use enterprise_facade_api::api::{
    adapter_response_single_execute_msg, AdaptedExecuteMsg, AdaptedMsg, AdapterResponse,
    AssetWhitelistParams, AssetWhitelistResponse, CastVoteMsg, ClaimsParams, ClaimsResponse,
    ComponentContractsResponse, CreateProposalMsg, CreateProposalWithDenomDepositMsg,
    CreateProposalWithTokenDepositMsg, DaoInfoResponse, DaoType, ExecuteProposalMsg,
    GovConfigFacade, ListMultisigMembersMsg, MemberInfoResponse, MemberVoteParams,
    MemberVoteResponse, MultisigMembersResponse, NftWhitelistParams, NftWhitelistResponse,
    Proposal, ProposalParams, ProposalResponse, ProposalStatus, ProposalStatusParams,
    ProposalStatusResponse, ProposalType, ProposalVotesParams, ProposalVotesResponse,
    ProposalsParams, ProposalsResponse, QueryMemberInfoMsg, StakeMsg, StakedNftsParams,
    StakedNftsResponse, TotalStakedAmountResponse, TreasuryAddressResponse, UnstakeMsg,
    UserStakeParams, UserStakeResponse, V2MigrationStage, V2MigrationStageResponse,
};
use enterprise_facade_api::error::DaoError::UnsupportedOperationForDaoType;
use enterprise_facade_api::error::EnterpriseFacadeError::Dao;
use enterprise_facade_api::error::{EnterpriseFacadeError, EnterpriseFacadeResult};
use enterprise_facade_common::facade::EnterpriseFacade;
use enterprise_governance_controller_api::api::{CreateProposalWithNftDepositMsg, ProposalAction};
use enterprise_outposts_api::api::{CrossChainTreasuriesParams, CrossChainTreasuriesResponse};
use enterprise_treasury_api::api::{
    HasIncompleteV2MigrationResponse, HasUnmovedStakesOrClaimsResponse,
};
use enterprise_versioning_api::api::{Version, VersionParams, VersionResponse};
use poll_engine::state::PollHelpers;
use poll_engine_api::api::{Poll, PollRejectionReason, PollStatus, VotingScheme};
use EnterpriseFacadeError::UnsupportedOperation;
use ExecuteV1Msg::ExecuteProposal;
use PollRejectionReason::{QuorumAndThresholdNotReached, QuorumNotReached, ThresholdNotReached};
use V2MigrationStage::MigrationNotStarted;

/// Facade implementation for v0.5.0 of Enterprise (pre-contract-rewrite).
pub struct EnterpriseFacadeV1 {
    pub enterprise_address: Addr,
}

impl EnterpriseFacade for EnterpriseFacadeV1 {
    fn query_treasury_address(
        &self,
        _: QueryContext,
    ) -> EnterpriseFacadeResult<TreasuryAddressResponse> {
        Ok(TreasuryAddressResponse {
            treasury_address: self.enterprise_address.clone(),
        })
    }

    fn query_dao_info(&self, qctx: QueryContext) -> EnterpriseFacadeResult<DaoInfoResponse> {
        let dao_info_v5: DaoInfoResponseV1 =
            self.query_enterprise_contract(qctx.deps, &DaoInfo {})?;

        let dao_version_from_code_version = Version {
            major: 0,
            minor: dao_info_v5.dao_code_version.u64(),
            patch: 0,
        };

        let veto_threshold = dao_info_v5
            .gov_config
            .veto_threshold
            .unwrap_or(dao_info_v5.gov_config.threshold);

        let gov_config = GovConfigFacade {
            quorum: dao_info_v5.gov_config.quorum,
            threshold: dao_info_v5.gov_config.threshold,
            veto_threshold,
            vote_duration: dao_info_v5.gov_config.vote_duration,
            unlocking_period: dao_info_v5.gov_config.unlocking_period,
            minimum_deposit: dao_info_v5.gov_config.minimum_deposit,
            allow_early_proposal_execution: dao_info_v5.gov_config.allow_early_proposal_execution,
        };

        Ok(DaoInfoResponse {
            creation_date: dao_info_v5.creation_date,
            metadata: dao_info_v5.metadata,
            gov_config,
            dao_council: dao_info_v5.dao_council,
            dao_type: dao_info_v5.dao_type,
            dao_membership_contract: dao_info_v5.dao_membership_contract.to_string(),
            enterprise_factory_contract: dao_info_v5.enterprise_factory_contract,
            funds_distributor_contract: dao_info_v5.funds_distributor_contract,
            dao_code_version: dao_info_v5.dao_code_version,
            dao_version: dao_version_from_code_version,
        })
    }

    fn query_component_contracts(
        &self,
        qctx: QueryContext,
    ) -> EnterpriseFacadeResult<ComponentContractsResponse> {
        let dao_info = self.query_dao_info(qctx)?;

        Ok(ComponentContractsResponse {
            enterprise_factory_contract: dao_info.enterprise_factory_contract,
            enterprise_contract: self.enterprise_address.clone(),
            funds_distributor_contract: dao_info.funds_distributor_contract,
            enterprise_governance_contract: None,
            enterprise_governance_controller_contract: None,
            enterprise_outposts_contract: None,
            enterprise_treasury_contract: None,
            membership_contract: None,
            council_membership_contract: None,
            attestation_contract: None,
        })
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
        let response: ProposalResponseV1 =
            self.query_enterprise_contract(qctx.deps, &v1_structs::QueryV1Msg::Proposal(params))?;

        let gov_config = self.query_dao_info(qctx.clone())?.gov_config;

        let fixed_response = self.fix_proposal_response(&qctx, response.into(), &gov_config)?;

        Ok(fixed_response)
    }

    fn query_proposals(
        &self,
        qctx: QueryContext,
        params: ProposalsParams,
    ) -> EnterpriseFacadeResult<ProposalsResponse> {
        let response: ProposalsResponseV1 =
            self.query_enterprise_contract(qctx.deps, &Proposals(params))?;

        let gov_config = self.query_dao_info(qctx.clone())?.gov_config;

        let fixed_responses = response
            .proposals
            .into_iter()
            .map(|proposal_response| {
                self.fix_proposal_response(&qctx, proposal_response.into(), &gov_config)
            })
            .collect::<EnterpriseFacadeResult<Vec<ProposalResponse>>>()?;

        Ok(ProposalsResponse {
            proposals: fixed_responses,
        })
    }

    fn query_proposal_status(
        &self,
        qctx: QueryContext,
        params: ProposalStatusParams,
    ) -> EnterpriseFacadeResult<ProposalStatusResponse> {
        let response = self.query_proposal(
            qctx,
            ProposalParams {
                proposal_id: params.proposal_id,
            },
        )?;

        Ok(ProposalStatusResponse {
            status: response.proposal_status,
            expires: response.proposal.expires,
            results: response.results,
        })
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
            &UserStake(UserStakeV1Params { user: params.user }),
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

    fn query_cross_chain_treasuries(
        &self,
        _: QueryContext,
        _: CrossChainTreasuriesParams,
    ) -> EnterpriseFacadeResult<CrossChainTreasuriesResponse> {
        Ok(CrossChainTreasuriesResponse { treasuries: vec![] })
    }

    fn query_has_incomplete_v2_migration(
        &self,
        _: QueryContext,
    ) -> EnterpriseFacadeResult<HasIncompleteV2MigrationResponse> {
        Ok(HasIncompleteV2MigrationResponse {
            has_incomplete_migration: false,
        })
    }

    fn query_has_unmoved_stakes_or_claims(
        &self,
        _: QueryContext,
    ) -> EnterpriseFacadeResult<HasUnmovedStakesOrClaimsResponse> {
        Ok(HasUnmovedStakesOrClaimsResponse {
            has_unmoved_stakes_or_claims: false,
        })
    }

    fn query_v2_migration_stage(
        &self,
        _: QueryContext,
    ) -> EnterpriseFacadeResult<V2MigrationStageResponse> {
        // for old DAOs, migration is not started yet
        Ok(V2MigrationStageResponse {
            stage: MigrationNotStarted,
        })
    }

    fn adapt_create_proposal(
        &self,
        qctx: QueryContext,
        params: CreateProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(adapter_response_single_execute_msg(
            self.enterprise_address.clone(),
            serde_json_wasm::to_string(&CreateProposal(
                self.map_create_proposal_msg(qctx.deps, params)?,
            ))?,
            vec![],
        ))
    }

    fn adapt_create_proposal_with_denom_deposit(
        &self,
        _: QueryContext,
        _: CreateProposalWithDenomDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Err(UnsupportedOperation)
    }

    fn adapt_create_proposal_with_token_deposit(
        &self,
        qctx: QueryContext,
        params: CreateProposalWithTokenDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        let dao_info = self.query_dao_info(qctx.clone())?;
        let dao_type = dao_info.dao_type;

        match dao_type {
            DaoType::Token => Ok(adapter_response_single_execute_msg(
                qctx.deps
                    .api
                    .addr_validate(&dao_info.dao_membership_contract)?,
                serde_json_wasm::to_string(&cw20::Cw20ExecuteMsg::Send {
                    contract: self.enterprise_address.to_string(),
                    amount: params.deposit_amount,
                    msg: to_json_binary(&Cw20HookV1Msg::CreateProposal(
                        self.map_create_proposal_msg(qctx.deps, params.create_proposal_msg)?,
                    ))?,
                })?,
                vec![],
            )),
            _ => Err(Dao(UnsupportedOperationForDaoType {
                dao_type: dao_type.to_string(),
            })),
        }
    }

    fn adapt_create_proposal_with_nft_deposit(
        &self,
        _: QueryContext,
        _: CreateProposalWithNftDepositMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Err(UnsupportedOperation)
    }

    fn adapt_create_council_proposal(
        &self,
        qctx: QueryContext,
        params: CreateProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(adapter_response_single_execute_msg(
            self.enterprise_address.clone(),
            serde_json_wasm::to_string(&CreateCouncilProposal(
                self.map_create_proposal_msg(qctx.deps, params)?,
            ))?,
            vec![],
        ))
    }

    fn adapt_cast_vote(
        &self,
        _: QueryContext,
        params: CastVoteMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(adapter_response_single_execute_msg(
            self.enterprise_address.clone(),
            serde_json_wasm::to_string(&CastVote(params))?,
            vec![],
        ))
    }

    fn adapt_cast_council_vote(
        &self,
        _: QueryContext,
        params: CastVoteMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(adapter_response_single_execute_msg(
            self.enterprise_address.clone(),
            serde_json_wasm::to_string(&CastCouncilVote(params))?,
            vec![],
        ))
    }

    fn adapt_execute_proposal(
        &self,
        _: QueryContext,
        params: ExecuteProposalMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(adapter_response_single_execute_msg(
            self.enterprise_address.clone(),
            serde_json_wasm::to_string(&ExecuteProposal(params))?,
            vec![],
        ))
    }

    fn adapt_stake(
        &self,
        qctx: QueryContext,
        params: StakeMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        match params {
            StakeMsg::Cw20(msg) => {
                let token_addr = self.query_dao_info(qctx.clone())?.dao_membership_contract;
                let msg = cw20::Cw20ExecuteMsg::Send {
                    contract: self.enterprise_address.to_string(),
                    amount: msg.amount,
                    msg: to_json_binary(&Cw20HookV1Msg::Stake {})?,
                };
                Ok(adapter_response_single_execute_msg(
                    qctx.deps.api.addr_validate(&token_addr)?,
                    serde_json_wasm::to_string(&msg)?,
                    vec![],
                ))
            }
            StakeMsg::Cw721(msg) => {
                let nft_addr = self.query_dao_info(qctx.clone())?.dao_membership_contract;
                let nft_addr = qctx.deps.api.addr_validate(&nft_addr)?;

                let stake_msg_binary = to_json_binary(&Cw721HookV1Msg::Stake {})?;

                let msgs = msg
                    .tokens
                    .into_iter()
                    .map(|token_id| cw721::Cw721ExecuteMsg::SendNft {
                        contract: self.enterprise_address.to_string(),
                        token_id,
                        msg: stake_msg_binary.clone(),
                    })
                    .map(|send_nft_msg| {
                        serde_json_wasm::to_string(&send_nft_msg).map(|send_nft_msg_json| {
                            AdaptedMsg::Execute(AdaptedExecuteMsg {
                                target_contract: nft_addr.clone(),
                                msg: send_nft_msg_json,
                                funds: vec![],
                            })
                        })
                    })
                    .collect::<serde_json_wasm::ser::Result<Vec<AdaptedMsg>>>()?;

                Ok(AdapterResponse { msgs })
            }
            StakeMsg::Denom(_) => Err(UnsupportedOperation),
        }
    }

    fn adapt_unstake(
        &self,
        _: QueryContext,
        params: UnstakeMsg,
    ) -> EnterpriseFacadeResult<AdapterResponse> {
        let params = match params {
            UnstakeMsg::Cw20(msg) => UnstakeV1Msg::Cw20(UnstakeCw20V1Msg { amount: msg.amount }),
            UnstakeMsg::Cw721(msg) => UnstakeV1Msg::Cw721(UnstakeCw721V1Msg { tokens: msg.tokens }),
            UnstakeMsg::Denom(_) => return Err(UnsupportedOperation),
        };
        Ok(adapter_response_single_execute_msg(
            self.enterprise_address.clone(),
            serde_json_wasm::to_string(&Unstake(params))?,
            vec![],
        ))
    }

    fn adapt_claim(&self, _: QueryContext) -> EnterpriseFacadeResult<AdapterResponse> {
        Ok(adapter_response_single_execute_msg(
            self.enterprise_address.clone(),
            serde_json_wasm::to_string(&Claim {})?,
            vec![],
        ))
    }
}

impl EnterpriseFacadeV1 {
    fn query_enterprise_contract<T: DeserializeOwned>(
        &self,
        deps: Deps,
        msg: &impl Serialize,
    ) -> EnterpriseFacadeResult<T> {
        Ok(deps
            .querier
            .query_wasm_smart(self.enterprise_address.to_string(), &msg)?)
    }

    fn resolve_in_progress_proposal_status(
        &self,
        response: &ProposalResponse,
        gov_config: &GovConfigFacade,
    ) -> EnterpriseFacadeResult<PollStatus> {
        // in reality, there were only AtTime expirations for proposals
        let ends_at = match response.proposal.expires {
            Expiration::AtTime(time) => time,
            _ => return Err(StdError::generic_err("invalid type of proposal expiry").into()),
        };

        let poll = Poll {
            id: response.proposal.id,
            proposer: response
                .proposal
                .proposer
                .clone()
                .unwrap_or(Addr::unchecked("")),
            deposit_amount: 0,
            label: response.proposal.title.clone(),
            description: response.proposal.description.clone(),
            scheme: VotingScheme::CoinVoting,
            status: PollStatus::InProgress { ends_at },
            started_at: response.proposal.started_at,
            ends_at,
            quorum: gov_config.quorum,
            threshold: gov_config.threshold,
            veto_threshold: Some(gov_config.veto_threshold),
            results: response.results.clone(),
        };

        let poll_status = poll.final_status(response.total_votes_available)?;

        Ok(poll_status)
    }

    fn map_proposal_action_to_v5(
        &self,
        deps: Deps,
        proposal_action: ProposalAction,
    ) -> StdResult<ProposalActionV1> {
        match proposal_action {
            ProposalAction::UpdateMetadata(msg) => Ok(UpdateMetadata(msg.into())),
            ProposalAction::UpdateGovConfig(msg) => Ok(UpdateGovConfig(msg.into())),
            ProposalAction::UpdateCouncil(msg) => Ok(UpdateCouncil(msg.into())),
            ProposalAction::UpdateAssetWhitelist(msg) => Ok(UpdateAssetWhitelist(msg.into())),
            ProposalAction::UpdateNftWhitelist(msg) => Ok(UpdateNftWhitelist(msg.into())),
            ProposalAction::RequestFundingFromDao(msg) => Ok(RequestFundingFromDao(msg.into())),
            ProposalAction::UpgradeDao(msg) => {
                let version_1_0_0 = Version {
                    major: 1,
                    minor: 0,
                    patch: 0,
                };
                let enterprise_versioning = ENTERPRISE_VERSIONING.load(deps.storage)?;

                if msg.new_version >= version_1_0_0 {
                    let version_1_0_0_info: VersionResponse = deps.querier.query_wasm_smart(
                        enterprise_versioning.to_string(),
                        &enterprise_versioning_api::msg::QueryMsg::Version(VersionParams {
                            version: version_1_0_0,
                        }),
                    )?;
                    // if we're migrating old DAO to rewritten structure, we first need to migrate to 1.0.0
                    Ok(UpgradeDao(UpgradeDaoV1Msg {
                        // send enterprise_treasury_code_id, since it takes the address of old enterprise contract
                        new_dao_code_id: version_1_0_0_info.version.enterprise_treasury_code_id,
                        migrate_msg: to_json_binary(&TreasuryV1_0_0MigrationMsg {
                            initial_submsgs_limit: None,
                        })?,
                    }))
                } else {
                    // we're migrating old DAO to a newer version of old DAO code
                    let version_info: VersionResponse = deps.querier.query_wasm_smart(
                        enterprise_versioning.to_string(),
                        &enterprise_versioning_api::msg::QueryMsg::Version(VersionParams {
                            version: msg.new_version,
                        }),
                    )?;
                    Ok(UpgradeDao(UpgradeDaoV1Msg {
                        new_dao_code_id: version_info.version.enterprise_code_id,
                        migrate_msg: to_json_binary(&Empty {})?,
                    }))
                }
            }
            ProposalAction::ExecuteMsgs(msg) => Ok(ExecuteMsgs(msg.into())),
            ProposalAction::ModifyMultisigMembership(msg) => {
                Ok(ModifyMultisigMembership(msg.into()))
            }
            ProposalAction::DistributeFunds(msg) => Ok(DistributeFunds(msg.into())),
            ProposalAction::UpdateMinimumWeightForRewards(msg) => {
                Ok(UpdateMinimumWeightForRewards(msg.into()))
            }
            ProposalAction::AddAttestation(_)
            | ProposalAction::RemoveAttestation { .. }
            | ProposalAction::DeployCrossChainTreasury(_)
            | ProposalAction::ExecuteTreasuryMsgs(_)
            | ProposalAction::ExecuteEnterpriseMsgs(_) => {
                Err(StdError::generic_err("unsupported proposal action"))
            }
        }
    }

    pub fn map_create_proposal_msg(
        &self,
        deps: Deps,
        msg: CreateProposalMsg,
    ) -> StdResult<CreateProposalV1Msg> {
        let proposal_actions = msg
            .proposal_actions
            .into_iter()
            .map(|it| self.map_proposal_action_to_v5(deps, it))
            .collect::<StdResult<Vec<ProposalActionV1>>>()?;

        Ok(CreateProposalV1Msg {
            title: msg.title,
            description: msg.description,
            proposal_actions,
        })
    }

    fn fix_proposal_response(
        &self,
        qctx: &QueryContext,
        response: ProposalResponse,
        gov_config: &GovConfigFacade,
    ) -> EnterpriseFacadeResult<ProposalResponse> {
        let status = match response.proposal_status {
            ProposalStatus::InProgress => {
                if response.proposal.expires.is_expired(&qctx.env.block) {
                    // proposal expired but stands as InProgress, let's resolve whether it passed or not
                    let poll_status =
                        self.resolve_in_progress_proposal_status(&response, gov_config)?;

                    match poll_status {
                        PollStatus::InProgress { .. } => return Err(StdError::generic_err("invalid state - resolved proposal status to 'in progress' after it ended").into()),
                        PollStatus::Passed { .. } => ProposalStatus::Passed,
                        PollStatus::Rejected { .. } => ProposalStatus::Rejected,
                    }
                } else {
                    // proposal still in progress, let's see if it can be executed early

                    let allows_early_ending = match response.proposal.proposal_type {
                        ProposalType::General => gov_config.allow_early_proposal_execution,
                        ProposalType::Council => true,
                    };

                    if allows_early_ending {
                        let poll_status =
                            self.resolve_in_progress_proposal_status(&response, gov_config)?;

                        match poll_status {
                            PollStatus::InProgress { .. } => ProposalStatus::InProgress,
                            PollStatus::Passed { .. } => ProposalStatus::InProgressCanExecuteEarly,
                            PollStatus::Rejected { reason } => match reason {
                                QuorumNotReached
                                | ThresholdNotReached
                                | QuorumAndThresholdNotReached => ProposalStatus::InProgress,
                                _ => ProposalStatus::InProgressCanExecuteEarly,
                            },
                        }
                    } else {
                        ProposalStatus::InProgress
                    }
                }
            }
            _ => response.proposal_status,
        };

        let fixed_response = ProposalResponse {
            proposal: Proposal {
                status: status.clone(),
                ..response.proposal
            },
            proposal_status: status,
            ..response
        };

        Ok(fixed_response)
    }
}
