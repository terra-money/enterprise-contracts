use crate::proposals::{
    get_proposal_actions, is_proposal_executed, set_proposal_executed, PROPOSAL_INFOS,
    TOTAL_DEPOSITS,
};
use crate::state::{State, COUNCIL_GOV_CONFIG, ENTERPRISE_CONTRACT, GOV_CONFIG, STATE};
use crate::validate::{
    apply_gov_config_changes, validate_dao_council, validate_dao_gov_config, validate_deposit,
    validate_modify_multisig_membership, validate_proposal_actions, validate_upgrade_dao,
};
use attestation_api::api::{HasUserSignedParams, HasUserSignedResponse};
use common::commons::ModifyValue::Change;
use common::cw::{Context, Pagination, QueryContext};
use cosmwasm_std::{
    coin, entry_point, from_binary, to_binary, wasm_execute, Addr, Binary, Coin, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult, SubMsg, Uint128, Uint64,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfoBase};
use cw_utils::Expiration;
use cw_utils::Expiration::Never;
use enterprise_governance_api::msg::ExecuteMsg::UpdateVotes;
use enterprise_governance_controller_api::api::ProposalAction::{
    AddAttestation, DistributeFunds, ExecuteMsgs, ModifyMultisigMembership, RemoveAttestation,
    RequestFundingFromDao, UpdateAssetWhitelist, UpdateCouncil, UpdateGovConfig, UpdateMetadata,
    UpdateMinimumWeightForRewards, UpdateNftWhitelist, UpgradeDao,
};
use enterprise_governance_controller_api::api::ProposalType::{Council, General};
use enterprise_governance_controller_api::api::{
    AddAttestationMsg, CastVoteMsg, CreateProposalMsg, DistributeFundsMsg, ExecuteMsgsMsg,
    ExecuteProposalMsg, GovConfig, GovConfigResponse, MemberVoteParams, MemberVoteResponse,
    ModifyMultisigMembershipMsg, Proposal, ProposalAction, ProposalActionType, ProposalDeposit,
    ProposalId, ProposalInfo, ProposalParams, ProposalResponse, ProposalStatus,
    ProposalStatusFilter, ProposalStatusParams, ProposalStatusResponse, ProposalType,
    ProposalVotesParams, ProposalVotesResponse, ProposalsParams, ProposalsResponse,
    RequestFundingFromDaoMsg, UpdateCouncilMsg, UpdateGovConfigMsg,
    UpdateMinimumWeightForRewardsMsg,
};
use enterprise_governance_controller_api::error::GovernanceControllerError::{
    CustomError, InvalidCosmosMessage, InvalidDepositType, NoDaoCouncil, NoSuchProposal,
    NoVotesAvailable, NoVotingPower, ProposalAlreadyExecuted, RestrictedUser, Std, Unauthorized,
    UnsupportedCouncilProposalAction, WrongProposalType,
};
use enterprise_governance_controller_api::error::GovernanceControllerResult;
use enterprise_governance_controller_api::msg::{
    Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use enterprise_governance_controller_api::response::{
    execute_cast_council_vote_response, execute_cast_vote_response,
    execute_create_council_proposal_response, execute_create_proposal_response,
    execute_execute_proposal_response, execute_weights_changed_response, instantiate_response,
    reply_create_poll_response,
};
use enterprise_protocol::api::{
    ComponentContractsResponse, DaoInfoResponse, DaoType, SetAttestationMsg, UpdateMetadataMsg,
    UpgradeDaoMsg,
};
use enterprise_protocol::msg::QueryMsg::{ComponentContracts, DaoInfo};
use enterprise_treasury_api::api::{SpendMsg, UpdateAssetWhitelistMsg, UpdateNftWhitelistMsg};
use enterprise_treasury_api::msg::ExecuteMsg::Spend;
use funds_distributor_api::api::{UpdateMinimumEligibleWeightMsg, UpdateUserWeightsMsg};
use funds_distributor_api::msg::Cw20HookMsg::Distribute;
use funds_distributor_api::msg::ExecuteMsg::DistributeNative;
use membership_common_api::api::{
    TotalWeightParams, TotalWeightResponse, UserWeightChange, UserWeightParams, UserWeightResponse,
    WeightsChangedMsg,
};
use multisig_membership_api::api::{SetMembersMsg, UpdateMembersMsg};
use multisig_membership_api::msg::ExecuteMsg::{SetMembers, UpdateMembers};
use poll_engine_api::api::{
    CastVoteParams, CreatePollParams, EndPollParams, Poll, PollId, PollParams, PollRejectionReason,
    PollResponse, PollStatus, PollStatusFilter, PollStatusResponse, PollVoterParams,
    PollVoterResponse, PollVotersParams, PollVotersResponse, PollsParams, PollsResponse,
    UpdateVotesParams, VotingScheme,
};
use poll_engine_api::error::PollError::PollInProgress;
use std::cmp::min;
use std::ops::{Add, Not, Sub};
use DaoType::{Denom, Multisig, Nft, Token};
use Expiration::{AtHeight, AtTime};
use PollRejectionReason::{IsVetoOutcome, QuorumNotReached};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise-governance-controller";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const CREATE_POLL_REPLY_ID: u64 = 1;
pub const END_POLL_REPLY_ID: u64 = 2;
pub const EXECUTE_PROPOSAL_ACTIONS_REPLY_ID: u64 = 3;

pub const DEFAULT_QUERY_LIMIT: u8 = 50;
pub const MAX_QUERY_LIMIT: u8 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> GovernanceControllerResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    STATE.save(
        deps.storage,
        &State {
            proposal_being_created: None,
            proposal_being_executed: None,
        },
    )?;

    let enterprise_contract = deps.api.addr_validate(&msg.enterprise_contract)?;
    ENTERPRISE_CONTRACT.save(deps.storage, &enterprise_contract)?;

    let dao_info: DaoInfoResponse = deps
        .querier
        .query_wasm_smart(enterprise_contract.to_string(), &DaoInfo {})?;

    validate_dao_gov_config(&dao_info.dao_type, &msg.gov_config)?;
    GOV_CONFIG.save(deps.storage, &msg.gov_config)?;

    let council_gov_config = validate_dao_council(deps.as_ref(), msg.council_gov_config)?;
    COUNCIL_GOV_CONFIG.save(deps.storage, &council_gov_config)?;

    for (proposal_id, proposal_info) in msg.proposal_infos.unwrap_or_default() {
        PROPOSAL_INFOS.save(deps.storage, proposal_id, &proposal_info)?;
    }

    Ok(instantiate_response())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> GovernanceControllerResult<Response> {
    let sender = info.sender.clone();
    let ctx = &mut Context { deps, env, info };
    match msg {
        ExecuteMsg::CreateProposal(msg) => create_proposal(ctx, msg, None, sender),
        ExecuteMsg::CreateCouncilProposal(msg) => create_council_proposal(ctx, msg),
        ExecuteMsg::CastVote(msg) => cast_vote(ctx, msg),
        ExecuteMsg::CastCouncilVote(msg) => cast_council_vote(ctx, msg),
        ExecuteMsg::ExecuteProposal(msg) => execute_proposal(ctx, msg),
        ExecuteMsg::ExecuteProposalActions(msg) => execute_proposal_actions(ctx, msg),
        ExecuteMsg::Receive(msg) => receive_cw20(ctx, msg),
        ExecuteMsg::WeightsChanged(msg) => weights_changed(ctx, msg),
    }
}

fn create_proposal(
    ctx: &mut Context,
    msg: CreateProposalMsg,
    deposit: Option<ProposalDeposit>,
    proposer: Addr,
) -> GovernanceControllerResult<Response> {
    unrestricted_users_only(ctx.deps.as_ref(), proposer.to_string())?;

    let gov_config = GOV_CONFIG.load(ctx.deps.storage)?;

    let qctx = QueryContext {
        deps: ctx.deps.as_ref(),
        env: ctx.env.clone(),
    };
    let user_available_votes = get_user_available_votes(qctx, proposer.clone())?;

    if user_available_votes.is_zero() {
        return Err(NoVotingPower);
    }

    validate_deposit(&gov_config, &deposit)?;
    validate_proposal_actions(
        ctx.deps.as_ref(),
        query_dao_type(ctx.deps.as_ref())?,
        &msg.proposal_actions,
    )?;

    let create_poll_submsg = create_poll(ctx, gov_config, msg, deposit, General, proposer)?;

    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

    Ok(
        execute_create_proposal_response(enterprise_contract.to_string())
            .add_submessage(create_poll_submsg),
    )
}

fn create_council_proposal(
    ctx: &mut Context,
    msg: CreateProposalMsg,
) -> GovernanceControllerResult<Response> {
    unrestricted_users_only(ctx.deps.as_ref(), ctx.info.sender.to_string())?;

    let dao_council = COUNCIL_GOV_CONFIG.load(ctx.deps.storage)?;

    match dao_council {
        None => Err(NoDaoCouncil),
        Some(dao_council) => {
            validate_proposal_actions(
                ctx.deps.as_ref(),
                query_dao_type(ctx.deps.as_ref())?,
                &msg.proposal_actions,
            )?;

            let member_weight = query_council_member_weight(
                ctx.deps.as_ref(),
                ctx.info.sender.clone().to_string(),
            )?;

            if member_weight.is_zero() {
                return Err(Unauthorized);
            }

            let allowed_actions = dao_council.allowed_proposal_action_types;

            // validate that proposal actions are allowed
            for proposal_action in &msg.proposal_actions {
                let proposal_action_type = to_proposal_action_type(proposal_action);
                if !allowed_actions.contains(&proposal_action_type) {
                    return Err(UnsupportedCouncilProposalAction {
                        action: proposal_action_type,
                    });
                }
            }

            let gov_config = GOV_CONFIG.load(ctx.deps.storage)?;

            let council_gov_config = GovConfig {
                quorum: dao_council.quorum,
                threshold: dao_council.threshold,
                ..gov_config
            };

            let create_poll_submsg = create_poll(
                ctx,
                council_gov_config,
                msg,
                None,
                Council,
                ctx.info.sender.clone(),
            )?;

            let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

            Ok(
                execute_create_council_proposal_response(enterprise_contract.to_string())
                    .add_submessage(create_poll_submsg),
            )
        }
    }
}

fn to_proposal_action_type(proposal_action: &ProposalAction) -> ProposalActionType {
    match proposal_action {
        UpdateMetadata(_) => ProposalActionType::UpdateMetadata,
        UpdateGovConfig(_) => ProposalActionType::UpdateGovConfig,
        UpdateCouncil(_) => ProposalActionType::UpdateCouncil,
        UpdateAssetWhitelist(_) => ProposalActionType::UpdateAssetWhitelist,
        UpdateNftWhitelist(_) => ProposalActionType::UpdateNftWhitelist,
        RequestFundingFromDao(_) => ProposalActionType::RequestFundingFromDao,
        UpgradeDao(_) => ProposalActionType::UpgradeDao,
        ExecuteMsgs(_) => ProposalActionType::ExecuteMsgs,
        ModifyMultisigMembership(_) => ProposalActionType::ModifyMultisigMembership,
        DistributeFunds(_) => ProposalActionType::DistributeFunds,
        UpdateMinimumWeightForRewards(_) => ProposalActionType::UpdateMinimumWeightForRewards,
        AddAttestation(_) => ProposalActionType::AddAttestation,
        RemoveAttestation {} => ProposalActionType::RemoveAttestation,
    }
}

fn create_poll(
    ctx: &mut Context,
    gov_config: GovConfig,
    msg: CreateProposalMsg,
    deposit: Option<ProposalDeposit>,
    proposal_type: ProposalType,
    proposer: Addr,
) -> GovernanceControllerResult<SubMsg> {
    let ends_at = ctx.env.block.time.plus_seconds(gov_config.vote_duration);

    let governance_contract = query_enterprise_governance_addr(ctx.deps.as_ref())?;
    let create_poll_submsg = SubMsg::reply_on_success(
        wasm_execute(
            governance_contract.to_string(),
            &enterprise_governance_api::msg::ExecuteMsg::CreatePoll(CreatePollParams {
                proposer: proposer.to_string(),
                deposit_amount: Uint128::zero(),
                label: msg.title,
                description: msg.description.unwrap_or_default(),
                scheme: VotingScheme::CoinVoting,
                ends_at,
                quorum: gov_config.quorum,
                threshold: gov_config.threshold,
                veto_threshold: gov_config.veto_threshold,
            }),
            vec![],
        )?,
        CREATE_POLL_REPLY_ID,
    );

    let state = STATE.load(ctx.deps.storage)?;
    if state.proposal_being_created.is_some() {
        return Err(CustomError {
            val: "Invalid state - found proposal being created when not expected".to_string(),
        });
    }
    STATE.save(
        ctx.deps.storage,
        &State {
            proposal_being_created: Some(ProposalInfo {
                proposal_type,
                executed_at: None,
                proposal_deposit: deposit,
                proposal_actions: msg.proposal_actions,
            }),
            ..state
        },
    )?;

    Ok(create_poll_submsg)
}

fn cast_vote(ctx: &mut Context, msg: CastVoteMsg) -> GovernanceControllerResult<Response> {
    unrestricted_users_only(ctx.deps.as_ref(), ctx.info.sender.to_string())?;

    let qctx = QueryContext::from(ctx.deps.as_ref(), ctx.env.clone());
    let user_available_votes = get_user_available_votes(qctx, ctx.info.sender.clone())?;

    if user_available_votes == Uint128::zero() {
        return Err(Unauthorized);
    }

    let proposal_info = PROPOSAL_INFOS
        .may_load(ctx.deps.storage, msg.proposal_id)?
        .ok_or(NoSuchProposal)?;

    if proposal_info.proposal_type != General {
        return Err(WrongProposalType);
    }

    let governance_contract = query_enterprise_governance_addr(ctx.deps.as_ref())?;

    let cast_vote_submessage = SubMsg::new(wasm_execute(
        governance_contract.to_string(),
        &enterprise_governance_api::msg::ExecuteMsg::CastVote(CastVoteParams {
            poll_id: msg.proposal_id.into(),
            outcome: msg.outcome,
            voter: ctx.info.sender.to_string(),
            amount: user_available_votes,
        }),
        vec![],
    )?);

    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

    Ok(execute_cast_vote_response(
        enterprise_contract.to_string(),
        msg.proposal_id,
        ctx.info.sender.to_string(),
        msg.outcome,
        user_available_votes,
    )
    .add_submessage(cast_vote_submessage))
}

fn cast_council_vote(ctx: &mut Context, msg: CastVoteMsg) -> GovernanceControllerResult<Response> {
    unrestricted_users_only(ctx.deps.as_ref(), ctx.info.sender.to_string())?;

    let dao_council = COUNCIL_GOV_CONFIG.load(ctx.deps.storage)?;

    match dao_council {
        None => Err(NoDaoCouncil),
        Some(_) => {
            let member_weight = query_council_member_weight(
                ctx.deps.as_ref(),
                ctx.info.sender.clone().to_string(),
            )?;

            if member_weight.is_zero() {
                return Err(Unauthorized);
            }

            let proposal_info = PROPOSAL_INFOS
                .may_load(ctx.deps.storage, msg.proposal_id)?
                .ok_or(NoSuchProposal)?;

            if proposal_info.proposal_type != Council {
                return Err(WrongProposalType);
            }

            let governance_contract = query_enterprise_governance_addr(ctx.deps.as_ref())?;

            let cast_vote_submessage = SubMsg::new(wasm_execute(
                governance_contract.to_string(),
                &enterprise_governance_api::msg::ExecuteMsg::CastVote(CastVoteParams {
                    poll_id: msg.proposal_id.into(),
                    outcome: msg.outcome,
                    voter: ctx.info.sender.to_string(),
                    amount: member_weight,
                }),
                vec![],
            )?);

            let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

            Ok(execute_cast_council_vote_response(
                enterprise_contract.to_string(),
                msg.proposal_id,
                ctx.info.sender.to_string(),
                msg.outcome,
                1u8.into(),
            )
            .add_submessage(cast_vote_submessage))
        }
    }
}

fn execute_proposal(
    ctx: &mut Context,
    msg: ExecuteProposalMsg,
) -> GovernanceControllerResult<Response> {
    unrestricted_users_only(ctx.deps.as_ref(), ctx.info.sender.to_string())?;

    let proposal_info = PROPOSAL_INFOS
        .may_load(ctx.deps.storage, msg.proposal_id)?
        .ok_or(NoSuchProposal)?;

    if proposal_info.executed_at.is_some() {
        return Err(ProposalAlreadyExecuted);
    }

    let submsgs = end_proposal(ctx, &msg, proposal_info.proposal_type.clone())?;

    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

    Ok(execute_execute_proposal_response(
        enterprise_contract.to_string(),
        msg.proposal_id,
        proposal_info.proposal_type,
    )
    .add_submessages(submsgs))
}

fn return_proposal_deposit_submsgs(
    deps: DepsMut,
    proposal_id: ProposalId,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let proposal_info = PROPOSAL_INFOS
        .may_load(deps.storage, proposal_id)?
        .ok_or(NoSuchProposal)?;

    return_deposit_submsgs(deps, proposal_info.proposal_deposit)
}

fn return_deposit_submsgs(
    deps: DepsMut,
    deposit: Option<ProposalDeposit>,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    match deposit {
        None => Ok(vec![]),
        Some(deposit) => {
            let membership_contract = query_membership_addr(deps.as_ref())?;

            let transfer_msg =
                Asset::cw20(membership_contract, deposit.amount).transfer_msg(deposit.depositor)?;

            TOTAL_DEPOSITS.update(deps.storage, |deposits| -> StdResult<Uint128> {
                Ok(deposits.sub(deposit.amount))
            })?;

            Ok(vec![SubMsg::new(transfer_msg)])
        }
    }
}

fn end_proposal(
    ctx: &mut Context,
    msg: &ExecuteProposalMsg,
    proposal_type: ProposalType,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let qctx = QueryContext::from(ctx.deps.as_ref(), ctx.env.clone());
    let poll = query_poll(&qctx, msg.proposal_id)?.poll;

    let ends_at = poll.ends_at;

    let total_available_votes = if ends_at <= ctx.env.block.time {
        general_total_available_votes(ctx.deps.as_ref(), AtTime(ends_at))?
    } else {
        general_total_available_votes(ctx.deps.as_ref(), Never {})?
    };

    if total_available_votes == Uint128::zero() {
        return Err(NoVotesAvailable);
    }

    let allow_early_ending = match proposal_type {
        General => {
            let gov_config = GOV_CONFIG.load(ctx.deps.storage)?;
            gov_config.allow_early_proposal_execution
        }
        Council => true,
    };

    let governance_contract = query_enterprise_governance_addr(ctx.deps.as_ref())?;
    let end_poll_submsg = SubMsg::reply_on_success(
        wasm_execute(
            governance_contract.to_string(),
            &enterprise_governance_api::msg::ExecuteMsg::EndPoll(EndPollParams {
                poll_id: msg.proposal_id.into(),
                maximum_available_votes: total_available_votes,
                error_if_already_ended: false,
                allow_early_ending,
            }),
            vec![],
        )?,
        END_POLL_REPLY_ID,
    );

    let state = STATE.load(ctx.deps.storage)?;
    if state.proposal_being_executed.is_some() {
        return Err(CustomError {
            val: "Invalid state: proposal being executed is present when not expected".to_string(),
        });
    }

    STATE.save(
        ctx.deps.storage,
        &State {
            proposal_being_executed: Some(msg.proposal_id),
            ..state
        },
    )?;

    Ok(vec![end_poll_submsg])
}

fn resolve_ended_proposal(
    ctx: &mut Context,
    proposal_id: ProposalId,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let qctx = QueryContext::from(ctx.deps.as_ref(), ctx.env.clone());
    let poll_status = query_poll_status(&qctx, proposal_id)?.status;

    let submsgs = match poll_status {
        PollStatus::InProgress { .. } => {
            return Err(PollInProgress {
                poll_id: proposal_id.into(),
            }
            .into())
        }
        PollStatus::Passed { .. } => {
            set_proposal_executed(ctx.deps.storage, proposal_id, ctx.env.block.clone())?;
            let execute_proposal_actions_msg = SubMsg::reply_always(
                wasm_execute(
                    ctx.env.contract.address.to_string(),
                    &ExecuteMsg::ExecuteProposalActions(ExecuteProposalMsg { proposal_id }),
                    vec![],
                )?,
                EXECUTE_PROPOSAL_ACTIONS_REPLY_ID,
            );
            let mut submsgs = return_proposal_deposit_submsgs(ctx.deps.branch(), proposal_id)?;

            submsgs.insert(0, execute_proposal_actions_msg);

            submsgs
        }
        PollStatus::Rejected { reason } => {
            set_proposal_executed(ctx.deps.storage, proposal_id, ctx.env.block.clone())?;

            let proposal_info = PROPOSAL_INFOS
                .may_load(ctx.deps.storage, proposal_id)?
                .ok_or(NoSuchProposal)?;

            match proposal_info.proposal_type {
                General => match reason {
                    QuorumNotReached | IsVetoOutcome => {
                        if let Some(deposit) = proposal_info.proposal_deposit {
                            TOTAL_DEPOSITS.update(
                                ctx.deps.storage,
                                |deposits| -> StdResult<Uint128> {
                                    Ok(deposits.sub(deposit.amount))
                                },
                            )?;
                        }
                        vec![]
                    }
                    // return deposits only if quorum reached and not vetoed
                    _ => return_proposal_deposit_submsgs(ctx.deps.branch(), proposal_id)?,
                },
                Council => vec![],
            }
        }
    };

    Ok(submsgs)
}

fn execute_proposal_actions(
    ctx: &mut Context,
    msg: ExecuteProposalMsg,
) -> GovernanceControllerResult<Response> {
    // only the DAO itself can execute this
    if ctx.info.sender != ctx.env.contract.address {
        return Err(Unauthorized);
    }

    let submsgs: Vec<SubMsg> = execute_proposal_actions_submsgs(ctx, msg.proposal_id)?;

    Ok(Response::new()
        .add_attribute("action", "execute_proposal_actions")
        .add_attribute("proposal_id", msg.proposal_id.to_string())
        .add_submessages(submsgs))
}

fn execute_proposal_actions_submsgs(
    ctx: &mut Context,
    proposal_id: ProposalId,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let proposal_actions =
        get_proposal_actions(ctx.deps.storage, proposal_id)?.ok_or(NoSuchProposal)?;

    let mut submsgs: Vec<SubMsg> = vec![];

    for proposal_action in proposal_actions {
        let mut actions = match proposal_action {
            UpdateMetadata(msg) => update_metadata(ctx.deps.branch(), msg)?,
            UpdateGovConfig(msg) => update_gov_config(ctx, msg)?,
            UpdateCouncil(msg) => update_council(ctx, msg)?,
            RequestFundingFromDao(msg) => execute_funding_from_dao(ctx.deps.branch(), msg)?,
            UpdateAssetWhitelist(msg) => update_asset_whitelist(ctx.deps.branch(), msg)?,
            UpdateNftWhitelist(msg) => update_nft_whitelist(ctx.deps.branch(), msg)?,
            UpgradeDao(msg) => upgrade_dao(ctx, msg)?,
            ExecuteMsgs(msg) => execute_msgs(msg)?,
            ModifyMultisigMembership(msg) => {
                modify_multisig_membership(ctx.deps.branch(), ctx.env.clone(), msg)?
            }
            DistributeFunds(msg) => distribute_funds(ctx, msg)?,
            UpdateMinimumWeightForRewards(msg) => update_minimum_weight_for_rewards(ctx, msg)?,
            AddAttestation(msg) => add_attestation(ctx, msg)?,
            RemoveAttestation {} => remove_attestation(ctx)?,
        };
        submsgs.append(&mut actions)
    }

    Ok(submsgs)
}

fn update_metadata(
    deps: DepsMut,
    msg: UpdateMetadataMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(deps.storage)?;

    let submsg = SubMsg::new(wasm_execute(
        enterprise_contract.to_string(),
        &enterprise_protocol::msg::ExecuteMsg::UpdateMetadata(msg),
        vec![],
    )?);

    Ok(vec![submsg])
}

fn execute_funding_from_dao(
    deps: DepsMut,
    msg: RequestFundingFromDaoMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let treasury_addr = query_enterprise_components(deps.as_ref())?.enterprise_treasury_contract;

    let submsg = SubMsg::new(wasm_execute(
        treasury_addr.to_string(),
        &Spend(SpendMsg {
            recipient: msg.recipient,
            assets: msg.assets,
        }),
        vec![],
    )?);

    Ok(vec![submsg])
}

fn update_gov_config(
    ctx: &mut Context,
    msg: UpdateGovConfigMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let gov_config = GOV_CONFIG.load(ctx.deps.storage)?;

    let updated_gov_config = apply_gov_config_changes(gov_config, &msg);

    validate_dao_gov_config(&query_dao_type(ctx.deps.as_ref())?, &updated_gov_config)?;

    GOV_CONFIG.save(ctx.deps.storage, &updated_gov_config)?;

    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;
    let dao_info: DaoInfoResponse = ctx
        .deps
        .querier
        .query_wasm_smart(enterprise_contract.to_string(), &DaoInfo {})?;
    let component_contracts: ComponentContractsResponse = ctx
        .deps
        .querier
        .query_wasm_smart(enterprise_contract.to_string(), &ComponentContracts {})?;

    let mut submsgs = vec![];

    if let Change(new_unlocking_period) = msg.unlocking_period {
        match dao_info.dao_type {
            Denom => submsgs.push(SubMsg::new(wasm_execute(
                component_contracts.membership_contract.to_string(),
                &denom_staking_api::msg::ExecuteMsg::UpdateUnlockingPeriod(
                    denom_staking_api::api::UpdateUnlockingPeriodMsg {
                        new_unlocking_period: Some(new_unlocking_period),
                    },
                ),
                vec![],
            )?)),
            Token => submsgs.push(SubMsg::new(wasm_execute(
                component_contracts.membership_contract.to_string(),
                &token_staking_api::msg::ExecuteMsg::UpdateUnlockingPeriod(
                    token_staking_api::api::UpdateUnlockingPeriodMsg {
                        new_unlocking_period: Some(new_unlocking_period),
                    },
                ),
                vec![],
            )?)),
            Nft => submsgs.push(SubMsg::new(wasm_execute(
                component_contracts.membership_contract.to_string(),
                &nft_staking_api::msg::ExecuteMsg::UpdateUnlockingPeriod(
                    nft_staking_api::api::UpdateUnlockingPeriodMsg {
                        new_unlocking_period: Some(new_unlocking_period),
                    },
                ),
                vec![],
            )?)),
            Multisig => {} // no-op
        }
    }

    Ok(submsgs)
}

fn update_council(
    ctx: &mut Context,
    msg: UpdateCouncilMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let dao_council = validate_dao_council(ctx.deps.as_ref(), msg.dao_council.clone())?;

    let dao_council_membership_contract = query_council_membership_addr(ctx.deps.as_ref())?;

    let new_members = msg
        .dao_council
        .map(|council| council.members)
        .unwrap_or_default()
        .into_iter()
        .map(|member| multisig_membership_api::api::UserWeight {
            user: member,
            weight: Uint128::one(),
        })
        .collect();

    COUNCIL_GOV_CONFIG.save(ctx.deps.storage, &dao_council)?;

    let submsg = SubMsg::new(wasm_execute(
        dao_council_membership_contract.to_string(),
        &SetMembers(SetMembersMsg { new_members }),
        vec![],
    )?);

    Ok(vec![submsg])
}

fn update_asset_whitelist(
    deps: DepsMut,
    msg: UpdateAssetWhitelistMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let treasury_addr = query_enterprise_components(deps.as_ref())?.enterprise_treasury_contract;

    let submsg = SubMsg::new(wasm_execute(
        treasury_addr.to_string(),
        &enterprise_treasury_api::msg::ExecuteMsg::UpdateAssetWhitelist(msg),
        vec![],
    )?);

    Ok(vec![submsg])
}

fn update_nft_whitelist(
    deps: DepsMut,
    msg: UpdateNftWhitelistMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let treasury_addr = query_enterprise_components(deps.as_ref())?.enterprise_treasury_contract;

    let submsg = SubMsg::new(wasm_execute(
        treasury_addr.to_string(),
        &enterprise_treasury_api::msg::ExecuteMsg::UpdateNftWhitelist(msg),
        vec![],
    )?);

    Ok(vec![submsg])
}

fn upgrade_dao(ctx: &mut Context, msg: UpgradeDaoMsg) -> GovernanceControllerResult<Vec<SubMsg>> {
    validate_upgrade_dao(ctx.deps.as_ref(), &msg)?;

    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

    let submsg = SubMsg::new(wasm_execute(
        enterprise_contract.to_string(),
        &enterprise_protocol::msg::ExecuteMsg::UpgradeDao(msg),
        vec![],
    )?);

    Ok(vec![submsg])
}

fn execute_msgs(msg: ExecuteMsgsMsg) -> GovernanceControllerResult<Vec<SubMsg>> {
    let mut submsgs: Vec<SubMsg> = vec![];
    for msg in msg.msgs {
        submsgs.push(SubMsg::new(
            serde_json_wasm::from_str::<CosmosMsg>(msg.as_str())
                .map_err(|_| InvalidCosmosMessage)?,
        ))
    }
    Ok(submsgs)
}

fn modify_multisig_membership(
    deps: DepsMut,
    _env: Env,
    msg: ModifyMultisigMembershipMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    validate_modify_multisig_membership(deps.as_ref(), query_dao_type(deps.as_ref())?, &msg)?;

    let membership_contract = query_membership_addr(deps.as_ref())?;

    let submsg = SubMsg::new(wasm_execute(
        membership_contract.to_string(),
        &UpdateMembers(UpdateMembersMsg {
            update_members: msg.edit_members,
        }),
        vec![],
    )?);

    Ok(vec![submsg])
}

fn distribute_funds(
    ctx: &mut Context,
    msg: DistributeFundsMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let mut native_funds: Vec<Coin> = vec![];
    let mut submsgs: Vec<SubMsg> = vec![];

    let funds_distributor =
        query_enterprise_components(ctx.deps.as_ref())?.funds_distributor_contract;

    for asset in msg.funds {
        match asset.info {
            AssetInfoBase::Native(denom) => native_funds.push(coin(asset.amount.u128(), denom)),
            AssetInfoBase::Cw20(_) => submsgs.push(SubMsg::new(
                asset.send_msg(funds_distributor.to_string(), to_binary(&Distribute {})?)?,
            )),
            AssetInfoBase::Cw1155(_, _) => {
                return Err(Std(StdError::generic_err(
                    "cw1155 assets are not supported at this time",
                )))
            }
            _ => return Err(Std(StdError::generic_err("unknown asset type"))),
        }
    }

    submsgs.push(SubMsg::new(wasm_execute(
        funds_distributor.to_string(),
        &DistributeNative {},
        native_funds,
    )?));

    Ok(submsgs)
}

fn update_minimum_weight_for_rewards(
    ctx: &mut Context,
    msg: UpdateMinimumWeightForRewardsMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let funds_distributor =
        query_enterprise_components(ctx.deps.as_ref())?.funds_distributor_contract;

    let submsg = SubMsg::new(wasm_execute(
        funds_distributor.to_string(),
        &funds_distributor_api::msg::ExecuteMsg::UpdateMinimumEligibleWeight(
            UpdateMinimumEligibleWeightMsg {
                minimum_eligible_weight: msg.minimum_weight_for_rewards,
            },
        ),
        vec![],
    )?);

    Ok(vec![submsg])
}

fn add_attestation(
    ctx: &mut Context,
    msg: AddAttestationMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

    let submsg = SubMsg::new(wasm_execute(
        enterprise_contract.to_string(),
        &enterprise_protocol::msg::ExecuteMsg::SetAttestation(SetAttestationMsg {
            attestation_text: msg.attestation_text,
        }),
        vec![],
    )?);

    Ok(vec![submsg])
}

fn remove_attestation(ctx: &mut Context) -> GovernanceControllerResult<Vec<SubMsg>> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

    let submsg = SubMsg::new(wasm_execute(
        enterprise_contract.to_string(),
        &enterprise_protocol::msg::ExecuteMsg::RemoveAttestation {},
        vec![],
    )?);

    Ok(vec![submsg])
}

pub fn receive_cw20(
    ctx: &mut Context,
    cw20_msg: Cw20ReceiveMsg,
) -> GovernanceControllerResult<Response> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::CreateProposal(msg)) => {
            // only membership CW20 contract can execute this message
            let dao_type = query_dao_type(ctx.deps.as_ref())?;

            let membership_contract = query_membership_addr(ctx.deps.as_ref())?;
            if dao_type != Token || ctx.info.sender != membership_contract {
                return Err(InvalidDepositType);
            }
            let depositor = ctx.deps.api.addr_validate(&cw20_msg.sender)?;
            let deposit = ProposalDeposit {
                depositor: depositor.clone(),
                amount: cw20_msg.amount,
            };
            create_proposal(ctx, msg, Some(deposit), depositor)
        }
        _ => Err(CustomError {
            val: "msg payload not recognized".to_string(),
        }),
    }
}

/// Callback invoked when membership weights change.
/// We need to update governance votes and funds distributor weights.
pub fn weights_changed(
    ctx: &mut Context,
    msg: WeightsChangedMsg,
) -> GovernanceControllerResult<Response> {
    let update_votes_submsgs = update_user_votes(ctx.deps.as_ref(), &msg.weight_changes)?;

    let new_user_weights = msg
        .weight_changes
        .into_iter()
        .map(
            |user_weight_change| funds_distributor_api::api::UserWeight {
                user: user_weight_change.user,
                weight: user_weight_change.new_weight,
            },
        )
        .collect();

    let update_funds_distributor_submsg = SubMsg::new(wasm_execute(
        query_enterprise_components(ctx.deps.as_ref())?
            .funds_distributor_contract
            .to_string(),
        &funds_distributor_api::msg::ExecuteMsg::UpdateUserWeights(UpdateUserWeightsMsg {
            new_user_weights,
        }),
        vec![],
    )?);

    Ok(execute_weights_changed_response()
        .add_submessages(update_votes_submsgs)
        .add_submessage(update_funds_distributor_submsg))
}

pub fn update_user_votes(
    deps: Deps,
    user_weight_changes: &Vec<UserWeightChange>,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let governance_contract = query_enterprise_governance_addr(deps)?;

    let mut update_votes_submsgs: Vec<SubMsg> = vec![];

    for user_weight_change in user_weight_changes {
        update_votes_submsgs.push(SubMsg::new(wasm_execute(
            governance_contract.to_string(),
            &UpdateVotes(UpdateVotesParams {
                voter: user_weight_change.user.clone(),
                new_amount: user_weight_change.new_weight,
            }),
            vec![],
        )?));
    }

    Ok(update_votes_submsgs)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> GovernanceControllerResult<Response> {
    match msg.id {
        CREATE_POLL_REPLY_ID => {
            let poll_id = parse_poll_id(msg)?;

            let state = STATE.load(deps.storage)?;

            let proposal_info = state.proposal_being_created.ok_or(CustomError {
                val: "Invalid state - missing proposal info".to_string(),
            })?;

            STATE.save(
                deps.storage,
                &State {
                    proposal_being_created: None,
                    ..state
                },
            )?;

            PROPOSAL_INFOS.save(deps.storage, poll_id, &proposal_info)?;

            if let Some(deposit) = proposal_info.proposal_deposit {
                TOTAL_DEPOSITS.update(deps.storage, |deposits| -> StdResult<Uint128> {
                    Ok(deposits.add(deposit.amount))
                })?;
            }

            Ok(reply_create_poll_response(poll_id))
        }
        END_POLL_REPLY_ID => {
            let info = MessageInfo {
                sender: env.contract.address.clone(),
                funds: vec![],
            };

            let ctx = &mut Context { deps, env, info };
            let state = STATE.load(ctx.deps.storage)?;

            let proposal_id = state.proposal_being_executed.ok_or(CustomError {
                val: "Invalid state - missing ID of proposal being executed".to_string(),
            })?;

            STATE.save(
                ctx.deps.storage,
                &State {
                    proposal_being_executed: None,
                    ..state
                },
            )?;

            let execute_submsgs = resolve_ended_proposal(ctx, proposal_id)?;

            Ok(Response::new().add_submessages(execute_submsgs))
        }
        EXECUTE_PROPOSAL_ACTIONS_REPLY_ID => {
            // no actions, regardless of the result
            Ok(Response::new())
        }
        _ => Err(Std(StdError::generic_err("No such reply ID found"))),
    }
}

fn parse_poll_id(msg: Reply) -> GovernanceControllerResult<PollId> {
    let events = msg
        .result
        .into_result()
        .map_err(|e| CustomError { val: e })?
        .events;
    let event = events
        .iter()
        .find(|event| {
            event
                .attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "create_poll")
        })
        .ok_or(CustomError {
            val: "Reply does not contain create_poll event".to_string(),
        })?;

    Uint64::try_from(
        event
            .attributes
            .iter()
            .find(|attr| attr.key == "poll_id")
            .ok_or(CustomError {
                val: "create_poll event does not contain poll ID".to_string(),
            })?
            .value
            .as_str(),
    )
    .map_err(|_| CustomError {
        val: "Invalid poll ID in reply".to_string(),
    })
    .map(|poll_id| poll_id.u64())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> GovernanceControllerResult<Binary> {
    let qctx = QueryContext::from(deps, env);

    let response = match msg {
        QueryMsg::GovConfig {} => to_binary(&query_gov_config(qctx)?)?,
        QueryMsg::Proposal(params) => to_binary(&query_proposal(qctx, params)?)?,
        QueryMsg::Proposals(params) => to_binary(&query_proposals(qctx, params)?)?,
        QueryMsg::ProposalStatus(params) => to_binary(&query_proposal_status(qctx, params)?)?,
        QueryMsg::MemberVote(params) => to_binary(&query_member_vote(qctx, params)?)?,
        QueryMsg::ProposalVotes(params) => to_binary(&query_proposal_votes(qctx, params)?)?,
    };
    Ok(response)
}

pub fn query_gov_config(qctx: QueryContext) -> GovernanceControllerResult<GovConfigResponse> {
    let gov_config = GOV_CONFIG.load(qctx.deps.storage)?;
    let dao_council_membership = query_council_membership_addr(qctx.deps)?;

    let membership_contract = query_membership_addr(qctx.deps)?;

    let council_gov_config = COUNCIL_GOV_CONFIG.load(qctx.deps.storage)?;

    Ok(GovConfigResponse {
        gov_config,
        council_gov_config,
        dao_membership_contract: membership_contract,
        dao_council_membership_contract: dao_council_membership,
    })
}

pub fn query_proposal(
    qctx: QueryContext,
    msg: ProposalParams,
) -> GovernanceControllerResult<ProposalResponse> {
    let poll = query_poll(&qctx, msg.proposal_id)?;

    let proposal = poll_to_proposal_response(qctx.deps, &qctx.env, &poll.poll)?;

    Ok(proposal)
}

fn query_poll(qctx: &QueryContext, poll_id: PollId) -> GovernanceControllerResult<PollResponse> {
    let governance_contract = query_enterprise_governance_addr(qctx.deps)?;

    let poll: PollResponse = qctx.deps.querier.query_wasm_smart(
        governance_contract.to_string(),
        &enterprise_governance_api::msg::QueryMsg::Poll(PollParams { poll_id }),
    )?;
    Ok(poll)
}

pub fn query_proposals(
    qctx: QueryContext,
    msg: ProposalsParams,
) -> GovernanceControllerResult<ProposalsResponse> {
    let governance_contract = query_enterprise_governance_addr(qctx.deps)?;

    let polls: PollsResponse = qctx.deps.querier.query_wasm_smart(
        governance_contract.to_string(),
        &enterprise_governance_api::msg::QueryMsg::Polls(PollsParams {
            filter: msg.filter.map(|filter| match filter {
                ProposalStatusFilter::InProgress => PollStatusFilter::InProgress,
                ProposalStatusFilter::Passed => PollStatusFilter::Passed,
                ProposalStatusFilter::Rejected => PollStatusFilter::Rejected,
            }),
            pagination: Pagination {
                start_after: msg.start_after.map(Uint64::from),
                end_at: None,
                limit: Some(
                    msg.limit
                        .map_or(DEFAULT_QUERY_LIMIT as u64, |limit| limit as u64)
                        .min(MAX_QUERY_LIMIT as u64),
                ),
                order_by: None,
            },
        }),
    )?;

    let proposals = polls
        .polls
        .into_iter()
        .filter_map(|poll| {
            let proposal_response = poll_to_proposal_response(qctx.deps, &qctx.env, &poll);
            // filthy hack: we do not store whether a poll is of type General or Council
            // we listed all polls in poll-engine, but only when we try to add remaining data
            // contained in this contract can we know what their type is and exclude them from
            // the results if they're not of the requested type
            if let Err(NoSuchProposal) = proposal_response {
                None
            } else {
                Some(proposal_response)
            }
        })
        .collect::<GovernanceControllerResult<Vec<ProposalResponse>>>()?;

    Ok(ProposalsResponse { proposals })
}

pub fn query_proposal_status(
    qctx: QueryContext,
    msg: ProposalStatusParams,
) -> GovernanceControllerResult<ProposalStatusResponse> {
    let poll_status = query_poll_status(&qctx, msg.proposal_id)?;

    let status = match poll_status.status {
        PollStatus::InProgress { .. } => ProposalStatus::InProgress,
        PollStatus::Passed { .. } => {
            if is_proposal_executed(qctx.deps.storage, msg.proposal_id)? {
                ProposalStatus::Executed
            } else {
                ProposalStatus::Passed
            }
        }
        PollStatus::Rejected { .. } => ProposalStatus::Rejected,
    };

    Ok(ProposalStatusResponse {
        status,
        expires: AtTime(poll_status.ends_at),
        results: poll_status.results,
    })
}

fn query_poll_status(
    qctx: &QueryContext,
    poll_id: PollId,
) -> GovernanceControllerResult<PollStatusResponse> {
    let governance_contract = query_enterprise_governance_addr(qctx.deps)?;
    let poll_status_response: PollStatusResponse = qctx.deps.querier.query_wasm_smart(
        governance_contract.to_string(),
        &enterprise_governance_api::msg::QueryMsg::PollStatus { poll_id },
    )?;

    Ok(poll_status_response)
}

fn poll_to_proposal_response(
    deps: Deps,
    env: &Env,
    poll: &Poll,
) -> GovernanceControllerResult<ProposalResponse> {
    let actions_opt = get_proposal_actions(deps.storage, poll.id)?;

    let actions = match actions_opt {
        None => return Err(NoSuchProposal),
        Some(actions) => actions,
    };

    let status = match poll.status {
        PollStatus::InProgress { .. } => ProposalStatus::InProgress,
        PollStatus::Passed { .. } => {
            if is_proposal_executed(deps.storage, poll.id)? {
                ProposalStatus::Executed
            } else {
                ProposalStatus::Passed
            }
        }
        PollStatus::Rejected { .. } => ProposalStatus::Rejected,
    };

    let info = PROPOSAL_INFOS.load(deps.storage, poll.id)?;

    let proposal = Proposal {
        proposal_type: info.proposal_type.clone(),
        id: poll.id,
        proposer: poll.proposer.clone(),
        title: poll.label.clone(),
        description: poll.description.clone(),
        status: status.clone(),
        started_at: poll.started_at,
        expires: AtTime(poll.ends_at),
        proposal_actions: actions,
    };

    let expiration = match info.executed_at {
        Some(executed_block) => match proposal.expires {
            AtHeight(height) => AtHeight(min(height, executed_block.height)),
            AtTime(time) => AtTime(min(time, executed_block.time)),
            Never {} => AtHeight(executed_block.height),
        },
        None => match proposal.expires {
            AtHeight(height) => {
                if env.block.height >= height {
                    AtHeight(height)
                } else {
                    Never {}
                }
            }
            AtTime(time) => {
                if env.block.time >= time {
                    AtTime(time)
                } else {
                    Never {}
                }
            }
            Never {} => Never {},
        },
    };

    let total_votes_available = total_available_votes(deps, expiration, info.proposal_type)?;

    Ok(ProposalResponse {
        proposal,
        proposal_status: status,
        results: poll.results.clone(),
        total_votes_available,
    })
}

fn total_available_votes(
    deps: Deps,
    expiration: Expiration,
    proposal_type: ProposalType,
) -> GovernanceControllerResult<Uint128> {
    match proposal_type {
        General => general_total_available_votes(deps, expiration),
        Council => query_council_total_weight(deps, expiration),
    }
}

fn general_total_available_votes(
    deps: Deps,
    expiration: Expiration,
) -> GovernanceControllerResult<Uint128> {
    let membership_contract = query_membership_addr(deps)?;

    let response: TotalWeightResponse = deps.querier.query_wasm_smart(
        membership_contract,
        &membership_common_api::msg::QueryMsg::TotalWeight(TotalWeightParams { expiration }),
    )?;
    Ok(response.total_weight)
}

pub fn query_member_vote(
    qctx: QueryContext,
    params: MemberVoteParams,
) -> GovernanceControllerResult<MemberVoteResponse> {
    let governance_contract = query_enterprise_governance_addr(qctx.deps)?;
    let vote: PollVoterResponse = qctx.deps.querier.query_wasm_smart(
        governance_contract.to_string(),
        &enterprise_governance_api::msg::QueryMsg::PollVoter(PollVoterParams {
            poll_id: params.proposal_id.into(),
            voter_addr: params.member,
        }),
    )?;

    Ok(MemberVoteResponse { vote: vote.vote })
}

pub fn query_proposal_votes(
    qctx: QueryContext,
    params: ProposalVotesParams,
) -> GovernanceControllerResult<ProposalVotesResponse> {
    let governance_contract = query_enterprise_governance_addr(qctx.deps)?;
    let poll_voters: PollVotersResponse = qctx.deps.querier.query_wasm_smart(
        governance_contract.to_string(),
        &enterprise_governance_api::msg::QueryMsg::PollVoters(PollVotersParams {
            poll_id: params.proposal_id,
            pagination: Pagination {
                start_after: params.start_after,
                end_at: None,
                limit: Some(
                    params
                        .limit
                        .map_or(DEFAULT_QUERY_LIMIT as u64, |limit| limit as u64)
                        .min(MAX_QUERY_LIMIT as u64),
                ),
                order_by: None,
            },
        }),
    )?;

    Ok(ProposalVotesResponse {
        votes: poll_voters.votes,
    })
}

fn get_user_available_votes(qctx: QueryContext, user: Addr) -> GovernanceControllerResult<Uint128> {
    let membership_contract = query_membership_addr(qctx.deps)?;

    let response: UserWeightResponse = qctx.deps.querier.query_wasm_smart(
        membership_contract.to_string(),
        &membership_common_api::msg::QueryMsg::UserWeight(UserWeightParams {
            user: user.to_string(),
        }),
    )?;

    Ok(response.weight)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> GovernanceControllerResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}

fn query_dao_type(deps: Deps) -> GovernanceControllerResult<DaoType> {
    let enterprise = ENTERPRISE_CONTRACT.load(deps.storage)?;

    let response: DaoInfoResponse = deps
        .querier
        .query_wasm_smart(enterprise.to_string(), &DaoInfo {})?;

    Ok(response.dao_type)
}

fn query_enterprise_components(
    deps: Deps,
) -> GovernanceControllerResult<ComponentContractsResponse> {
    let enterprise = ENTERPRISE_CONTRACT.load(deps.storage)?;

    let response: ComponentContractsResponse = deps
        .querier
        .query_wasm_smart(enterprise.to_string(), &ComponentContracts {})?;

    Ok(response)
}

fn query_enterprise_governance_addr(deps: Deps) -> GovernanceControllerResult<Addr> {
    Ok(query_enterprise_components(deps)?.enterprise_governance_contract)
}

fn query_membership_addr(deps: Deps) -> GovernanceControllerResult<Addr> {
    Ok(query_enterprise_components(deps)?.membership_contract)
}

fn query_council_membership_addr(deps: Deps) -> GovernanceControllerResult<Addr> {
    Ok(query_enterprise_components(deps)?.council_membership_contract)
}

fn query_council_member_weight(deps: Deps, member: String) -> GovernanceControllerResult<Uint128> {
    let dao_council_membership = query_council_membership_addr(deps)?;

    let member_weight: UserWeightResponse = deps.querier.query_wasm_smart(
        dao_council_membership.to_string(),
        &multisig_membership_api::msg::QueryMsg::UserWeight(UserWeightParams { user: member }),
    )?;

    Ok(member_weight.weight)
}

fn query_council_total_weight(
    deps: Deps,
    expiration: Expiration,
) -> GovernanceControllerResult<Uint128> {
    let dao_council_membership = query_council_membership_addr(deps)?;

    let member_weight: UserWeightResponse = deps.querier.query_wasm_smart(
        dao_council_membership.to_string(),
        &multisig_membership_api::msg::QueryMsg::TotalWeight(TotalWeightParams { expiration }),
    )?;

    Ok(member_weight.weight)
}

/// Checks whether the user should be restricted from participating, i.e. there is an attestation
/// that they didn't sign.
fn is_restricted_user(deps: Deps, user: String) -> GovernanceControllerResult<bool> {
    let attestation_contract = query_enterprise_components(deps)?.attestation_contract;

    match attestation_contract {
        None => Ok(false),
        Some(attestation_addr) => {
            let has_user_signed_response: HasUserSignedResponse = deps.querier.query_wasm_smart(
                attestation_addr.to_string(),
                &attestation_api::msg::QueryMsg::HasUserSigned(HasUserSignedParams { user }),
            )?;

            Ok(has_user_signed_response.has_signed.not())
        }
    }
}

fn unrestricted_users_only(deps: Deps, user: String) -> GovernanceControllerResult<()> {
    if is_restricted_user(deps, user)? {
        return Err(RestrictedUser);
    }

    Ok(())
}
