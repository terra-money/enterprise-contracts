use crate::proposals::{
    get_proposal_actions, is_proposal_executed, set_proposal_executed, ProposalInfo,
    PROPOSAL_INFOS, TOTAL_DEPOSITS,
};
use crate::state::{State, DAO_COUNCIL, DAO_TYPE, ENTERPRISE_CONTRACT, GOV_CONFIG, STATE};
use crate::validate::{
    apply_gov_config_changes, normalize_asset_whitelist, validate_dao_council, validate_deposit,
    validate_modify_multisig_membership, validate_proposal_actions,
};
use common::cw::{Context, Pagination, QueryContext};
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{
    coin, entry_point, from_binary, to_binary, wasm_execute, wasm_instantiate, Addr, Binary,
    BlockInfo, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, StdResult, Storage, SubMsg, Timestamp, Uint128, Uint64, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::{Cw20Coin, Cw20ReceiveMsg, Logo, MinterResponse};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};
use cw_storage_plus::Bound;
use cw_utils::{parse_reply_instantiate_data, Duration, Expiration};
use enterprise_governance_controller_api::api::ModifyValue::Change;
use enterprise_governance_controller_api::api::ProposalAction::{
    DistributeFunds, ExecuteMsgs, ModifyMultisigMembership, RequestFundingFromDao,
    UpdateAssetWhitelist, UpdateCouncil, UpdateGovConfig, UpdateMetadata,
    UpdateMinimumWeightForRewards, UpdateNftWhitelist, UpgradeDao,
};
use enterprise_governance_controller_api::api::ProposalType::{Council, General};
use enterprise_governance_controller_api::api::{
    CastVoteMsg, CreateProposalMsg, DistributeFundsMsg, ExecuteMsgsMsg, ExecuteProposalMsg,
    GovConfig, GovConfigResponse, MemberVoteParams, MemberVoteResponse,
    ModifyMultisigMembershipMsg, Proposal, ProposalAction, ProposalActionType, ProposalDeposit,
    ProposalId, ProposalParams, ProposalResponse, ProposalStatus, ProposalStatusFilter,
    ProposalStatusParams, ProposalStatusResponse, ProposalType, ProposalVotesParams,
    ProposalVotesResponse, ProposalsParams, ProposalsResponse, RequestFundingFromDaoMsg,
    UpdateAssetWhitelistMsg, UpdateCouncilMsg, UpdateGovConfigMsg, UpdateMetadataMsg,
    UpdateMinimumWeightForRewardsMsg, UpdateNftWhitelistMsg, UpgradeDaoMsg,
};
use enterprise_governance_controller_api::error::GovernanceControllerError::{
    CustomError, InvalidCosmosMessage, InvalidDepositType, NoDaoCouncil, NoSuchProposal,
    NoVotesAvailable, ProposalAlreadyExecuted, Std, Unauthorized, UnsupportedCouncilProposalAction,
    WrongProposalType,
};
use enterprise_governance_controller_api::error::{
    GovernanceControllerError, GovernanceControllerResult,
};
use enterprise_governance_controller_api::msg::{
    Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use enterprise_protocol::api::DaoType;
use funds_distributor_api::api::{
    UpdateMinimumEligibleWeightMsg, UpdateUserWeightsMsg, UserWeight,
};
use funds_distributor_api::msg::Cw20HookMsg::Distribute;
use funds_distributor_api::msg::ExecuteMsg::DistributeNative;
use poll_engine_api::api::{
    CastVoteParams, CreatePollParams, EndPollParams, Poll, PollId, PollParams, PollRejectionReason,
    PollResponse, PollStatus, PollStatusFilter, PollStatusResponse, PollVoterParams,
    PollVoterResponse, PollVotersParams, PollVotersResponse, PollsParams, PollsResponse,
    UpdateVotesParams, VotingScheme,
};
use poll_engine_api::error::PollError::PollInProgress;
use std::cmp::min;
use std::ops::{Add, Not, Sub};
use Duration::{Height, Time};
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

    // TODO: validate gov config?
    GOV_CONFIG.save(deps.storage, &msg.gov_config)?;

    ENTERPRISE_CONTRACT.save(
        deps.storage,
        &deps.api.addr_validate(&msg.enterprise_contract)?,
    )?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> GovernanceControllerResult<Response> {
    let sender = info.sender.clone();
    let mut ctx = Context { deps, env, info };
    match msg {
        ExecuteMsg::CreateProposal(msg) => create_proposal(&mut ctx, msg, None, sender),
        ExecuteMsg::CreateCouncilProposal(msg) => create_council_proposal(&mut ctx, msg),
        ExecuteMsg::CastVote(msg) => cast_vote(&mut ctx, msg),
        ExecuteMsg::CastCouncilVote(msg) => cast_council_vote(&mut ctx, msg),
        ExecuteMsg::ExecuteProposal(msg) => execute_proposal(&mut ctx, msg),
        ExecuteMsg::ExecuteProposalActions(msg) => execute_proposal_actions(&mut ctx, msg),
        ExecuteMsg::Receive(msg) => receive_cw20(&mut ctx, msg),
    }
}

fn create_proposal(
    ctx: &mut Context,
    msg: CreateProposalMsg,
    deposit: Option<ProposalDeposit>,
    proposer: Addr,
) -> GovernanceControllerResult<Response> {
    let gov_config = GOV_CONFIG.load(ctx.deps.storage)?;

    validate_deposit(&gov_config, &deposit)?;
    validate_proposal_actions(ctx.deps.as_ref(), &msg.proposal_actions)?;

    let _dao_type = DAO_TYPE.load(ctx.deps.storage)?;

    let create_poll_submsg = create_poll(ctx, gov_config, msg, deposit, General, proposer)?;

    let response = Response::new()
        .add_attribute("action", "create_proposal")
        .add_attribute("dao_address", ctx.env.contract.address.to_string())
        .add_submessage(create_poll_submsg);

    // TODO: check if member can create a proposal, fail if they can't (not an NFT owner, multisig or council member, etc)

    Ok(response)
}

fn create_council_proposal(
    ctx: &mut Context,
    msg: CreateProposalMsg,
) -> GovernanceControllerResult<Response> {
    let dao_council = DAO_COUNCIL.load(ctx.deps.storage)?;

    match dao_council {
        None => Err(NoDaoCouncil),
        Some(dao_council) => {
            validate_proposal_actions(ctx.deps.as_ref(), &msg.proposal_actions)?;

            let proposer = ctx.info.sender.clone();

            if !dao_council.members.contains(&proposer) {
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

            Ok(Response::new()
                .add_attribute("action", "create_council_proposal")
                .add_attribute("dao_address", ctx.env.contract.address.to_string())
                .add_submessage(create_poll_submsg))
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

    // TODO: enterprise query for addr
    let governance_contract = Addr::unchecked("governance");
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

    // TODO: enterprise query for addr
    let governance_contract = Addr::unchecked("governance");

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

    Ok(Response::new()
        .add_attribute("action", "cast_vote")
        .add_attribute("dao_address", ctx.env.contract.address.to_string())
        .add_attribute("proposal_id", msg.proposal_id.to_string())
        .add_attribute("voter", ctx.info.sender.clone().to_string())
        .add_attribute("outcome", msg.outcome.to_string())
        .add_attribute("amount", user_available_votes.to_string())
        .add_submessage(cast_vote_submessage))
}

fn cast_council_vote(ctx: &mut Context, msg: CastVoteMsg) -> GovernanceControllerResult<Response> {
    let dao_council = DAO_COUNCIL.load(ctx.deps.storage)?;

    match dao_council {
        None => Err(NoDaoCouncil),
        Some(dao_council) => {
            if !dao_council.members.contains(&ctx.info.sender) {
                return Err(Unauthorized);
            }

            let proposal_info = PROPOSAL_INFOS
                .may_load(ctx.deps.storage, msg.proposal_id)?
                .ok_or(NoSuchProposal)?;

            if proposal_info.proposal_type != Council {
                return Err(WrongProposalType);
            }

            // TODO: enterprise query for addr
            let governance_contract = Addr::unchecked("governance");

            let cast_vote_submessage = SubMsg::new(wasm_execute(
                governance_contract.to_string(),
                &enterprise_governance_api::msg::ExecuteMsg::CastVote(CastVoteParams {
                    poll_id: msg.proposal_id.into(),
                    outcome: msg.outcome,
                    voter: ctx.info.sender.to_string(),
                    amount: Uint128::one(),
                }),
                vec![],
            )?);

            Ok(Response::new()
                .add_attribute("action", "cast_vote")
                .add_attribute("dao_address", ctx.env.contract.address.to_string())
                .add_attribute("proposal_id", msg.proposal_id.to_string())
                .add_attribute("voter", ctx.info.sender.clone().to_string())
                .add_attribute("outcome", msg.outcome.to_string())
                .add_attribute("amount", 1u8.to_string())
                .add_submessage(cast_vote_submessage))
        }
    }
}

fn execute_proposal(
    ctx: &mut Context,
    msg: ExecuteProposalMsg,
) -> GovernanceControllerResult<Response> {
    let proposal_info = PROPOSAL_INFOS
        .may_load(ctx.deps.storage, msg.proposal_id)?
        .ok_or(NoSuchProposal)?;

    if proposal_info.executed_at.is_some() {
        return Err(ProposalAlreadyExecuted);
    }

    let submsgs = end_proposal(ctx, &msg, proposal_info.proposal_type.clone())?;

    Ok(Response::new()
        .add_submessages(submsgs)
        .add_attribute("action", "execute_proposal")
        .add_attribute("dao_address", ctx.env.contract.address.to_string())
        .add_attribute("proposal_id", msg.proposal_id.to_string())
        .add_attribute("proposal_type", proposal_info.proposal_type.to_string()))
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
            // TODO: enterprise query for addr
            let membership_contract = Addr::unchecked("membership");

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
    let _poll = query_poll(&qctx, msg.proposal_id)?;

    // TODO: load total available votes from membership contract
    let total_available_votes = Uint128::zero();

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

    // TODO: enterprise query for addr
    let governance_contract = Addr::unchecked("governance");
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

// TODO: tests
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
            RequestFundingFromDao(msg) => execute_funding_from_dao(msg)?,
            UpdateAssetWhitelist(msg) => update_asset_whitelist(ctx.deps.branch(), msg)?,
            UpdateNftWhitelist(msg) => update_nft_whitelist(ctx.deps.branch(), msg)?,
            UpgradeDao(msg) => upgrade_dao(ctx.env.clone(), msg)?,
            ExecuteMsgs(msg) => execute_msgs(msg)?,
            ModifyMultisigMembership(msg) => {
                modify_multisig_membership(ctx.deps.branch(), ctx.env.clone(), msg)?
            }
            DistributeFunds(msg) => distribute_funds(ctx, msg)?,
            UpdateMinimumWeightForRewards(msg) => update_minimum_weight_for_rewards(ctx, msg)?,
        };
        submsgs.append(&mut actions)
    }

    Ok(submsgs)
}

fn update_metadata(
    _deps: DepsMut,
    _msg: UpdateMetadataMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    // TODO: send out msg

    Ok(vec![])
}

fn execute_funding_from_dao(
    _msg: RequestFundingFromDaoMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    // TODO: send out msg

    Ok(vec![])
}

fn update_gov_config(
    ctx: &mut Context,
    msg: UpdateGovConfigMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let gov_config = GOV_CONFIG.load(ctx.deps.storage)?;

    let updated_gov_config = apply_gov_config_changes(gov_config, &msg);

    // TODO: validate new gov config

    GOV_CONFIG.save(ctx.deps.storage, &updated_gov_config)?;

    Ok(vec![])
}

fn update_council(
    _ctx: &mut Context,
    _msg: UpdateCouncilMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    // TODO: send out msg

    Ok(vec![])
}

fn update_asset_whitelist(
    _deps: DepsMut,
    _msg: UpdateAssetWhitelistMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    // TODO: send out msg

    Ok(vec![])
}

fn update_nft_whitelist(
    _deps: DepsMut,
    _msg: UpdateNftWhitelistMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    // TODO: send out msg

    Ok(vec![])
}

fn upgrade_dao(_env: Env, _msg: UpgradeDaoMsg) -> GovernanceControllerResult<Vec<SubMsg>> {
    // TODO: send out msg
    Ok(vec![])
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
    validate_modify_multisig_membership(deps.as_ref(), &msg)?;

    // TODO: send out msg to membership contract

    Ok(vec![])
}

// TODO: tests
fn distribute_funds(
    _ctx: &mut Context,
    msg: DistributeFundsMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let mut native_funds: Vec<Coin> = vec![];
    let mut submsgs: Vec<SubMsg> = vec![];

    // TODO: enterprise query for addr
    let funds_distributor = Addr::unchecked("funds_distr");

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
    _ctx: &mut Context,
    msg: UpdateMinimumWeightForRewardsMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    // TODO: enterprise query for addr
    let funds_distributor = Addr::unchecked("funds_distr");

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

pub fn receive_cw20(
    ctx: &mut Context,
    cw20_msg: Cw20ReceiveMsg,
) -> GovernanceControllerResult<Response> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::CreateProposal(msg)) => {
            // only membership CW20 contract can execute this message
            let dao_type = DAO_TYPE.load(ctx.deps.storage)?;
            // TODO: enterprise query for addr
            let membership_contract = Addr::unchecked("membership");
            if dao_type != DaoType::Token || ctx.info.sender != membership_contract {
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

pub fn update_user_votes(
    _deps: Deps,
    user: Addr,
    new_amount: Uint128,
) -> GovernanceControllerResult<SubMsg> {
    // TODO: enterprise query for addr
    let governance_contract = Addr::unchecked("governance");

    let update_votes_submsg = SubMsg::new(wasm_execute(
        governance_contract.to_string(),
        &enterprise_governance_api::msg::ExecuteMsg::UpdateVotes(UpdateVotesParams {
            voter: user.to_string(),
            new_amount,
        }),
        vec![],
    )?);

    Ok(update_votes_submsg)
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

            Ok(Response::new().add_attribute("proposal_id", poll_id.to_string()))
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

            Ok(Response::new()
                .add_attribute("action", "execute_proposal")
                .add_attribute("dao_address", ctx.env.contract.address.to_string())
                .add_attribute("proposal_id", proposal_id.to_string())
                .add_submessages(execute_submsgs))
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

    Ok(GovConfigResponse {
        gov_config,
        dao_membership_contract: Addr::unchecked("membership_contract"), // TODO: read from state
        dao_council_contract: Addr::unchecked("council_contract"),       // TODO: read from state
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
    // TODO: get from enterprise contract
    let governance_contract = Addr::unchecked("governance");

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
    // TODO: enterprise query for addr
    let governance_contract = Addr::unchecked("governance");

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
        expires: Expiration::AtTime(poll_status.ends_at),
        results: poll_status.results,
    })
}

fn query_poll_status(
    qctx: &QueryContext,
    poll_id: PollId,
) -> GovernanceControllerResult<PollStatusResponse> {
    // TODO: enterprise query for addr
    let governance_contract = Addr::unchecked("governance");
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
        expires: Expiration::AtTime(poll.ends_at),
        proposal_actions: actions,
    };

    let dao_type = DAO_TYPE.load(deps.storage)?;

    let total_votes_available = match info.proposal_type {
        General => match info.executed_at {
            Some(block) => match proposal.expires {
                Expiration::AtHeight(height) => total_available_votes_at_height(
                    dao_type,
                    deps.storage,
                    min(height, block.height),
                )?,
                Expiration::AtTime(time) => {
                    total_available_votes_at_time(dao_type, deps.storage, min(time, block.time))?
                }
                Expiration::Never { .. } => {
                    // TODO: introduce a different structure to eliminate this branch
                    total_available_votes_at_height(dao_type, deps.storage, block.height)?
                }
            },
            None => match proposal.expires {
                Expiration::AtHeight(height) => {
                    if env.block.height >= height {
                        total_available_votes_at_height(dao_type, deps.storage, height)?
                    } else {
                        current_total_available_votes(dao_type, deps.storage)?
                    }
                }
                Expiration::AtTime(time) => {
                    if env.block.time >= time {
                        total_available_votes_at_time(dao_type, deps.storage, time)?
                    } else {
                        current_total_available_votes(dao_type, deps.storage)?
                    }
                }
                Expiration::Never { .. } => current_total_available_votes(dao_type, deps.storage)?,
            },
        },
        Council => {
            let dao_council = DAO_COUNCIL.load(deps.storage)?;

            match dao_council {
                None => return Err(NoDaoCouncil),
                Some(dao_council) => Uint128::from(dao_council.members.len() as u128),
            }
        }
    };

    Ok(ProposalResponse {
        proposal,
        proposal_status: status,
        results: poll.results.clone(),
        total_votes_available,
    })
}

fn total_available_votes_at_height(
    _dao_type: DaoType,
    _store: &dyn Storage,
    _height: u64,
) -> StdResult<Uint128> {
    // TODO: query membership contract
    Ok(Uint128::zero())
}

fn total_available_votes_at_time(
    _dao_type: DaoType,
    _store: &dyn Storage,
    _time: Timestamp,
) -> StdResult<Uint128> {
    // TODO: query membership contract
    Ok(Uint128::zero())
}

fn current_total_available_votes(_dao_type: DaoType, _store: &dyn Storage) -> StdResult<Uint128> {
    // TODO: query membership contract
    Ok(Uint128::zero())
}

pub fn query_member_vote(
    qctx: QueryContext,
    params: MemberVoteParams,
) -> GovernanceControllerResult<MemberVoteResponse> {
    // TODO: enterprise query for addr
    let governance_contract = Addr::unchecked("governance");
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
    // TODO: enterprise query for addr
    let governance_contract = Addr::unchecked("governance");
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

fn get_user_available_votes(
    _qctx: QueryContext,
    _user: Addr,
) -> GovernanceControllerResult<Uint128> {
    // TODO: query membership contract
    let user_available_votes = Uint128::zero();

    Ok(user_available_votes)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> GovernanceControllerResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
