use crate::ibc_hooks::{
    ibc_hooks_msg_to_ics_proxy_contract, IcsProxyCallback, IcsProxyCallbackType,
    IcsProxyInstantiateMsg, ICS_PROXY_CALLBACKS, ICS_PROXY_CALLBACK_LAST_ID,
};
use crate::proposals::{
    get_proposal_actions, is_proposal_executed, set_proposal_executed, PROPOSAL_INFOS,
};
use crate::state::{State, COUNCIL_GOV_CONFIG, ENTERPRISE_CONTRACT, GOV_CONFIG, STATE};
use crate::validate::{
    apply_gov_config_changes, validate_dao_council, validate_dao_gov_config, validate_deposit,
    validate_modify_multisig_membership, validate_proposal_actions, validate_unlocking_period,
    validate_upgrade_dao,
};
use common::commons::ModifyValue::Change;
use common::cw::{Context, Pagination, QueryContext};
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::{
    entry_point, from_binary, to_binary, wasm_execute, Addr, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Reply, Response, StdError, StdResult, SubMsg, SubMsgResponse, SubMsgResult,
    Uint128, Uint64, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ExecuteMsg::TransferNft;
use cw721::Cw721QueryMsg::OwnerOf;
use cw721::{Approval, OwnerOfResponse};
use cw_asset::{Asset, AssetInfoUnchecked};
use cw_utils::Expiration::Never;
use cw_utils::{parse_reply_instantiate_data, Expiration};
use denom_staking_api::api::DenomConfigResponse;
use denom_staking_api::msg::QueryMsg::DenomConfig;
use enterprise_governance_api::msg::ExecuteMsg::UpdateVotes;
use enterprise_governance_controller_api::api::ProposalAction::{
    AddAttestation, DistributeFunds, ExecuteMsgs, ModifyMultisigMembership, RemoveAttestation,
    RequestFundingFromDao, UpdateAssetWhitelist, UpdateCouncil, UpdateGovConfig, UpdateMetadata,
    UpdateMinimumWeightForRewards, UpdateNftWhitelist, UpgradeDao,
};
use enterprise_governance_controller_api::api::ProposalType::{Council, General};
use enterprise_governance_controller_api::api::{
    AddAttestationMsg, CastVoteMsg, ConfigResponse, CreateProposalMsg,
    CreateProposalWithNftDepositMsg, CrossChainMsgSpec, DeployCrossChainTreasuryMsg,
    DistributeFundsMsg, ExecuteMsgReplyCallbackMsg, ExecuteMsgsMsg, ExecuteProposalMsg, GovConfig,
    GovConfigResponse, MemberVoteParams, MemberVoteResponse, ModifyMultisigMembershipMsg, Proposal,
    ProposalAction, ProposalActionType, ProposalDeposit, ProposalDepositAsset, ProposalId,
    ProposalInfo, ProposalParams, ProposalResponse, ProposalStatus, ProposalStatusFilter,
    ProposalStatusParams, ProposalStatusResponse, ProposalType, ProposalVotesParams,
    ProposalVotesResponse, ProposalsParams, ProposalsResponse, RemoteTreasuryTarget,
    RequestFundingFromDaoMsg, UpdateAssetWhitelistProposalActionMsg, UpdateCouncilMsg,
    UpdateGovConfigMsg, UpdateMinimumWeightForRewardsMsg, UpdateNftWhitelistProposalActionMsg,
};
use enterprise_governance_controller_api::error::GovernanceControllerError::{
    CustomError, Dao, DuplicateNftDeposit, InvalidCosmosMessage, InvalidDepositType,
    NoCrossChainDeploymentForGivenChainId, NoDaoCouncil, NoSuchProposal, NoVotesAvailable,
    NoVotingPower, ProposalAlreadyExecuted, RestrictedUser, Std, Unauthorized,
    UnsupportedCouncilProposalAction, UnsupportedOperationForDaoType, WrongProposalType,
};
use enterprise_governance_controller_api::error::GovernanceControllerResult;
use enterprise_governance_controller_api::msg::{
    Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use enterprise_governance_controller_api::response::{
    execute_cast_council_vote_response, execute_cast_vote_response,
    execute_create_council_proposal_response, execute_create_proposal_response,
    execute_execute_msg_reply_callback_response, execute_execute_proposal_response,
    execute_weights_changed_response, instantiate_response, reply_create_poll_response,
};
use enterprise_protocol::api::{
    AddCrossChainProxyMsg, AddCrossChainTreasuryMsg, ComponentContractsResponse,
    CrossChainDeploymentsParams, CrossChainDeploymentsResponse, DaoInfoResponse, DaoType,
    IsRestrictedUserParams, IsRestrictedUserResponse, SetAttestationMsg, UpdateMetadataMsg,
    UpgradeDaoMsg,
};
use enterprise_protocol::error::DaoError::TreasuryAlreadyExistsForChainId;
use enterprise_protocol::msg::ExecuteMsg::{AddCrossChainProxy, AddCrossChainTreasury};
use enterprise_protocol::msg::QueryMsg::{
    ComponentContracts, CrossChainDeployments, DaoInfo, IsRestrictedUser,
};
use enterprise_treasury_api::api::{SpendMsg, UpdateAssetWhitelistMsg, UpdateNftWhitelistMsg};
use enterprise_treasury_api::msg::ExecuteMsg::Spend;
use funds_distributor_api::api::{UpdateMinimumEligibleWeightMsg, UpdateUserWeightsMsg};
use membership_common_api::api::{
    TotalWeightParams, TotalWeightResponse, UserWeightChange, UserWeightParams, UserWeightResponse,
    WeightsChangedMsg,
};
use multisig_membership_api::api::{SetMembersMsg, UpdateMembersMsg};
use multisig_membership_api::msg::ExecuteMsg::{SetMembers, UpdateMembers};
use nft_staking_api::api::{NftConfigResponse, NftTokenId};
use nft_staking_api::msg::QueryMsg::NftConfig;
use poll_engine_api::api::{
    CastVoteParams, CreatePollParams, EndPollParams, Poll, PollId, PollParams, PollRejectionReason,
    PollResponse, PollStatus, PollStatusFilter, PollStatusResponse, PollVoterParams,
    PollVoterResponse, PollVotersParams, PollVotersResponse, PollsParams, PollsResponse,
    UpdateVotesParams, VotingScheme,
};
use poll_engine_api::error::PollError::PollInProgress;
use std::cmp::min;
use std::collections::HashSet;
use token_staking_api::api::TokenConfigResponse;
use token_staking_api::msg::QueryMsg::TokenConfig;
use DaoType::{Denom, Multisig, Nft, Token};
use Expiration::{AtHeight, AtTime};
use PollRejectionReason::{IsVetoOutcome, QuorumNotReached};
use ProposalAction::DeployCrossChainTreasury;
use WasmMsg::Instantiate;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise-governance-controller";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const CREATE_POLL_REPLY_ID: u64 = 1;
pub const END_POLL_REPLY_ID: u64 = 2;
pub const EXECUTE_PROPOSAL_ACTIONS_REPLY_ID: u64 = 3;

pub const INSTANTIATE_CROSS_CHAIN_ICS_PROXY_CALLBACK_ID: u32 = 1001;
pub const INSTANTIATE_CROSS_CHAIN_TREASURY_CALLBACK_ID: u32 = 1002;

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

    validate_dao_gov_config(&msg.dao_type, &msg.gov_config)?;
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
    let ctx = &mut Context { deps, env, info };
    match msg {
        ExecuteMsg::CreateProposal(msg) => determine_deposit_and_create_proposal(ctx, msg),
        ExecuteMsg::CreateProposalWithNftDeposit(msg) => create_proposal_with_nft_deposit(ctx, msg),
        ExecuteMsg::CreateCouncilProposal(msg) => create_council_proposal(ctx, msg),
        ExecuteMsg::CastVote(msg) => cast_vote(ctx, msg),
        ExecuteMsg::CastCouncilVote(msg) => cast_council_vote(ctx, msg),
        ExecuteMsg::ExecuteProposal(msg) => execute_proposal(ctx, msg),
        ExecuteMsg::ExecuteProposalActions(msg) => execute_proposal_actions(ctx, msg),
        ExecuteMsg::Receive(msg) => receive_cw20(ctx, msg),
        ExecuteMsg::WeightsChanged(msg) => weights_changed(ctx, msg),
        ExecuteMsg::ExecuteMsgReplyCallback(msg) => execute_msg_reply_callback(ctx, msg),
    }
}

fn determine_deposit_and_create_proposal(
    ctx: &mut Context,
    msg: CreateProposalMsg,
) -> GovernanceControllerResult<Response> {
    let dao_type = query_dao_type(ctx.deps.as_ref())?;

    let proposer = ctx.info.sender.clone();

    let deposit = match dao_type {
        Denom => {
            let dao_denom_config = query_dao_denom_config(ctx.deps.as_ref())?;

            // find if the message contains funds that match the DAO's denom
            let dao_denom_from_funds = ctx
                .info
                .funds
                .iter()
                .find(|coin| coin.denom == dao_denom_config.denom);

            dao_denom_from_funds.map(|coin| ProposalDeposit {
                depositor: proposer.clone(),
                asset: ProposalDepositAsset::Denom {
                    denom: coin.denom.clone(),
                    amount: coin.amount,
                },
            })
        }
        Token | Nft | Multisig => None,
    };

    create_proposal(ctx, msg, deposit, proposer)
}

fn create_proposal_with_nft_deposit(
    ctx: &mut Context,
    msg: CreateProposalWithNftDepositMsg,
) -> GovernanceControllerResult<Response> {
    let dao_type = query_dao_type(ctx.deps.as_ref())?;

    if dao_type != Nft {
        return Err(UnsupportedOperationForDaoType {
            dao_type: dao_type.to_string(),
        });
    }

    assert_no_duplicate_nft_deposits(&msg.deposit_tokens)?;

    let dao_nft_config = query_dao_nft_config(ctx.deps.as_ref())?;

    let proposer = ctx.info.sender.clone();

    let mut transfer_deposit_tokens_msgs: Vec<SubMsg> = vec![];

    for token_id in &msg.deposit_tokens {
        // ensure that proposer is either an owner or is approved for every token being deposited
        if !can_deposit_nft(
            ctx,
            dao_nft_config.nft_contract.to_string(),
            proposer.clone(),
            token_id.to_string(),
        )? {
            return Err(Unauthorized);
        }

        // add a msg to transfer the NFT token being deposited to this contract
        // this assumes that this contract was given approval for the token, otherwise this fails
        transfer_deposit_tokens_msgs.push(SubMsg::new(wasm_execute(
            dao_nft_config.nft_contract.to_string(),
            &TransferNft {
                recipient: ctx.env.contract.address.to_string(),
                token_id: token_id.to_string(),
            },
            vec![],
        )?));
    }

    let nft_deposit = ProposalDeposit {
        depositor: proposer.clone(),
        asset: ProposalDepositAsset::Cw721 {
            nft_addr: dao_nft_config.nft_contract,
            tokens: msg.deposit_tokens,
        },
    };

    let create_proposal_response =
        create_proposal(ctx, msg.create_proposal_msg, Some(nft_deposit), proposer)?;

    Ok(create_proposal_response.add_submessages(transfer_deposit_tokens_msgs))
}

fn assert_no_duplicate_nft_deposits(tokens: &Vec<NftTokenId>) -> GovernanceControllerResult<()> {
    let mut token_set: HashSet<NftTokenId> = HashSet::new();

    for token in tokens {
        let newly_inserted = token_set.insert(token.to_string());
        if !newly_inserted {
            return Err(DuplicateNftDeposit);
        }
    }

    Ok(())
}

fn can_deposit_nft(
    ctx: &Context,
    nft_contract: String,
    proposer: Addr,
    token_id: NftTokenId,
) -> GovernanceControllerResult<bool> {
    let owner_response: OwnerOfResponse = ctx.deps.querier.query_wasm_smart(
        nft_contract,
        &OwnerOf {
            token_id,
            include_expired: Some(false),
        },
    )?;

    let owner = ctx.deps.api.addr_validate(&owner_response.owner)?;

    // only owners and users with an approval can deposit the NFT
    let can_deposit_nft = owner == proposer
        || has_nft_approval(ctx.deps.as_ref(), proposer, owner_response.approvals)?;

    Ok(can_deposit_nft)
}

fn has_nft_approval(
    deps: Deps,
    user: Addr,
    approvals: Vec<Approval>,
) -> GovernanceControllerResult<bool> {
    for approval in approvals {
        let spender = deps.api.addr_validate(&approval.spender)?;
        if spender == user {
            return Ok(true);
        }
    }
    Ok(false)
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
        DeployCrossChainTreasury(_) => ProposalActionType::DeployCrossChainTreasury,
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

    if let Some(deposit) = proposal_info.proposal_deposit {
        send_proposal_deposit_to(deposit.asset, deposit.depositor)
    } else {
        Ok(vec![])
    }
}

fn send_proposal_deposit_to(
    deposit_asset: ProposalDepositAsset,
    recipient: Addr,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let transfer_msgs = match deposit_asset {
        ProposalDepositAsset::Denom { denom, amount } => {
            vec![SubMsg::new(
                Asset::native(denom, amount).transfer_msg(recipient)?,
            )]
        }
        ProposalDepositAsset::Cw20 { token_addr, amount } => {
            vec![SubMsg::new(
                Asset::cw20(token_addr, amount).transfer_msg(recipient)?,
            )]
        }
        ProposalDepositAsset::Cw721 { nft_addr, tokens } => tokens
            .into_iter()
            .map(|token_id| {
                wasm_execute(
                    nft_addr.to_string(),
                    &TransferNft {
                        recipient: recipient.to_string(),
                        token_id,
                    },
                    vec![],
                )
                .map(SubMsg::new)
            })
            .collect::<StdResult<Vec<SubMsg>>>()?,
    };

    Ok(transfer_msgs)
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
        total_available_votes(ctx.deps.as_ref(), AtTime(ends_at), proposal_type.clone())?
    } else {
        total_available_votes(ctx.deps.as_ref(), Never {}, proposal_type.clone())?
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
                            // confiscate the deposit by sending it to treasury
                            let treasury_contract =
                                query_enterprise_treasury_addr(ctx.deps.as_ref())?;
                            send_proposal_deposit_to(deposit.asset, treasury_contract)?
                        } else {
                            vec![]
                        }
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
            RequestFundingFromDao(msg) => {
                execute_funding_from_dao(ctx.deps.branch(), ctx.env.clone(), msg)?
            }
            UpdateAssetWhitelist(msg) => {
                update_asset_whitelist(ctx.deps.branch(), ctx.env.clone(), msg)?
            }
            UpdateNftWhitelist(msg) => {
                update_nft_whitelist(ctx.deps.branch(), ctx.env.clone(), msg)?
            }
            UpgradeDao(msg) => upgrade_dao(ctx, msg)?,
            ExecuteMsgs(msg) => execute_msgs(msg)?,
            ModifyMultisigMembership(msg) => {
                modify_multisig_membership(ctx.deps.branch(), ctx.env.clone(), msg)?
            }
            DistributeFunds(msg) => distribute_funds(ctx, msg)?,
            UpdateMinimumWeightForRewards(msg) => update_minimum_weight_for_rewards(ctx, msg)?,
            AddAttestation(msg) => add_attestation(ctx, msg)?,
            RemoveAttestation {} => remove_attestation(ctx)?,
            DeployCrossChainTreasury(msg) => deploy_cross_chain_treasury(ctx, msg)?,
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
    env: Env,
    msg: RequestFundingFromDaoMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let submsg = execute_treasury_msg(
        deps,
        env,
        Spend(SpendMsg {
            recipient: msg.recipient,
            assets: msg.assets,
        }),
        msg.remote_treasury_target,
    )?;

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

    let dao_type = query_dao_type(ctx.deps.as_ref())?;

    let membership_contract = query_membership_addr(ctx.deps.as_ref())?;

    let mut submsgs = vec![];

    if let Change(new_unlocking_period) = msg.unlocking_period {
        validate_unlocking_period(updated_gov_config, new_unlocking_period)?;

        match dao_type {
            Denom => submsgs.push(SubMsg::new(wasm_execute(
                membership_contract.to_string(),
                &denom_staking_api::msg::ExecuteMsg::UpdateUnlockingPeriod(
                    denom_staking_api::api::UpdateUnlockingPeriodMsg {
                        new_unlocking_period: Some(new_unlocking_period),
                    },
                ),
                vec![],
            )?)),
            Token => submsgs.push(SubMsg::new(wasm_execute(
                membership_contract.to_string(),
                &token_staking_api::msg::ExecuteMsg::UpdateUnlockingPeriod(
                    token_staking_api::api::UpdateUnlockingPeriodMsg {
                        new_unlocking_period: Some(new_unlocking_period),
                    },
                ),
                vec![],
            )?)),
            Nft => submsgs.push(SubMsg::new(wasm_execute(
                membership_contract.to_string(),
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
    env: Env,
    msg: UpdateAssetWhitelistProposalActionMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let update_asset_whitelist_msg =
        enterprise_treasury_api::msg::ExecuteMsg::UpdateAssetWhitelist(UpdateAssetWhitelistMsg {
            add: msg.add,
            remove: msg.remove,
        });

    let submsg = execute_treasury_msg(
        deps,
        env,
        update_asset_whitelist_msg,
        msg.remote_treasury_target,
    )?;

    Ok(vec![submsg])
}

fn update_nft_whitelist(
    deps: DepsMut,
    env: Env,
    msg: UpdateNftWhitelistProposalActionMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let update_nft_whitelist_msg =
        enterprise_treasury_api::msg::ExecuteMsg::UpdateNftWhitelist(UpdateNftWhitelistMsg {
            add: msg.add,
            remove: msg.remove,
        });

    let submsg = execute_treasury_msg(
        deps,
        env,
        update_nft_whitelist_msg,
        msg.remote_treasury_target,
    )?;

    Ok(vec![submsg])
}

fn execute_treasury_msg(
    deps: DepsMut,
    env: Env,
    treasury_msg: enterprise_treasury_api::msg::ExecuteMsg,
    remote_treasury_target: Option<RemoteTreasuryTarget>,
) -> GovernanceControllerResult<SubMsg> {
    match remote_treasury_target {
        Some(remote_treasury_target) => {
            let response = query_enterprise_cross_chain_deployments(
                deps.as_ref(),
                remote_treasury_target.cross_chain_msg_spec.chain_id.clone(),
            )?;

            let proxy_addr = response
                .proxy_addr
                .ok_or(NoCrossChainDeploymentForGivenChainId)?;
            let treasury_addr = response
                .treasury_addr
                .ok_or(NoCrossChainDeploymentForGivenChainId)?;

            ibc_hooks_msg_to_ics_proxy_contract(
                &env,
                wasm_execute(treasury_addr, &treasury_msg, vec![])?.into(),
                proxy_addr,
                remote_treasury_target.cross_chain_msg_spec,
                None,
            )
        }
        None => {
            let treasury_addr = query_enterprise_treasury_addr(deps.as_ref())?;

            Ok(SubMsg::new(wasm_execute(
                treasury_addr.to_string(),
                &treasury_msg,
                vec![],
            )?))
        }
    }
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
    let enterprise_components = query_enterprise_components(ctx.deps.as_ref())?;

    let submsg = SubMsg::new(wasm_execute(
        enterprise_components
            .enterprise_treasury_contract
            .to_string(),
        &enterprise_treasury_api::msg::ExecuteMsg::DistributeFunds(
            enterprise_treasury_api::api::DistributeFundsMsg {
                funds: msg.funds,
                funds_distributor_contract: enterprise_components
                    .funds_distributor_contract
                    .to_string(),
            },
        ),
        vec![],
    )?);

    Ok(vec![submsg])
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

fn deploy_cross_chain_treasury(
    ctx: &mut Context,
    msg: DeployCrossChainTreasuryMsg,
) -> GovernanceControllerResult<Vec<SubMsg>> {
    let deployments_response = query_enterprise_cross_chain_deployments(
        ctx.deps.as_ref(),
        msg.cross_chain_msg_spec.chain_id.clone(),
    )?;

    if deployments_response.treasury_addr.is_some() {
        return Err(Dao(TreasuryAlreadyExistsForChainId));
    }

    match deployments_response.proxy_addr {
        Some(proxy_contract) => {
            // there is already a proxy contract owned by this DAO,
            // so we just go ahead and instantiate the treasury

            let instantiate_treasury_msg = instantiate_remote_treasury(
                ctx.deps.branch(),
                ctx.env.clone(),
                msg.enterprise_treasury_code_id,
                proxy_contract,
                msg.asset_whitelist,
                msg.nft_whitelist,
                msg.cross_chain_msg_spec,
            )?;

            Ok(vec![instantiate_treasury_msg])
        }
        None => {
            // there is no proxy contract owned by this DAO on the given chain,
            // so we go ahead and instantiate the proxy first

            // TODO: should we disallow multiple ongoing instantiations for the same chain?

            let global_proxy_address = ctx.deps.api.addr_canonicalize(&msg.chain_global_proxy)?;

            let callback_id = ICS_PROXY_CALLBACK_LAST_ID
                .may_load(ctx.deps.storage)?
                .unwrap_or_default()
                + 1;
            ICS_PROXY_CALLBACK_LAST_ID.save(ctx.deps.storage, &callback_id)?;

            ICS_PROXY_CALLBACKS.save(
                ctx.deps.storage,
                callback_id,
                &IcsProxyCallback {
                    chain_id: msg.cross_chain_msg_spec.chain_id.clone(),
                    proxy_addr: global_proxy_address.clone(),
                    callback_type: IcsProxyCallbackType::InstantiateProxy {
                        deploy_treasury_msg: Box::new(msg.clone()),
                    },
                },
            )?;

            let instantiate_proxy_msg = ibc_hooks_msg_to_ics_proxy_contract(
                &ctx.env,
                Wasm(Instantiate {
                    admin: None, // TODO: consider adding an admin, otherwise we'll be redeploying proxies
                    code_id: msg.ics_proxy_code_id,
                    msg: to_binary(&IcsProxyInstantiateMsg {
                        allow_cross_chain_msgs: true,
                        owner: Some(ctx.env.contract.address.to_string()),
                        whitelist: Some(vec![ctx.env.contract.address.to_string()]),
                        msgs: None,
                    })?,
                    funds: vec![],
                    label: "Proxy contract".to_string(),
                }),
                global_proxy_address.to_string(),
                msg.cross_chain_msg_spec,
                Some(callback_id),
            )?;
            Ok(vec![instantiate_proxy_msg])
        }
    }
}

fn instantiate_remote_treasury(
    deps: DepsMut,
    env: Env,
    enterprise_treasury_code_id: u64,
    proxy_contract: String,
    asset_whitelist: Option<Vec<AssetInfoUnchecked>>,
    nft_whitelist: Option<Vec<String>>,
    cross_chain_msg_spec: CrossChainMsgSpec,
) -> GovernanceControllerResult<SubMsg> {
    let callback_id = ICS_PROXY_CALLBACK_LAST_ID
        .may_load(deps.storage)?
        .unwrap_or_default()
        + 1;
    ICS_PROXY_CALLBACK_LAST_ID.save(deps.storage, &callback_id)?;

    let proxy_address = deps.api.addr_canonicalize(&proxy_contract)?;

    // TODO: should we disallow multiple ongoing instantiations for the same chain?

    ICS_PROXY_CALLBACKS.save(
        deps.storage,
        callback_id,
        &IcsProxyCallback {
            chain_id: cross_chain_msg_spec.chain_id.clone(),
            proxy_addr: proxy_address,
            callback_type: IcsProxyCallbackType::InstantiateTreasury {},
        },
    )?;

    let instantiate_treasury_msg = ibc_hooks_msg_to_ics_proxy_contract(
        &env,
        Wasm(Instantiate {
            admin: Some(proxy_contract.clone()),
            code_id: enterprise_treasury_code_id,
            msg: to_binary(&enterprise_treasury_api::msg::InstantiateMsg {
                admin: proxy_contract.clone(),
                asset_whitelist,
                nft_whitelist,
            })?,
            funds: vec![],
            label: "Proxy treasury".to_string(),
        }),
        proxy_contract,
        cross_chain_msg_spec,
        Some(callback_id),
    )?;
    Ok(instantiate_treasury_msg)
}

pub fn receive_cw20(
    ctx: &mut Context,
    cw20_msg: Cw20ReceiveMsg,
) -> GovernanceControllerResult<Response> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::CreateProposal(msg)) => {
            // only membership CW20 contract can execute this message
            let dao_type = query_dao_type(ctx.deps.as_ref())?;

            let token_contract = query_dao_token_config(ctx.deps.as_ref())?.token_contract;

            if dao_type != Token || ctx.info.sender != token_contract {
                return Err(InvalidDepositType);
            }
            let depositor = ctx.deps.api.addr_validate(&cw20_msg.sender)?;
            let deposit = ProposalDeposit {
                depositor: depositor.clone(),
                asset: ProposalDepositAsset::Cw20 {
                    token_addr: token_contract,
                    amount: cw20_msg.amount,
                },
            };
            create_proposal(ctx, msg, Some(deposit), depositor)
        }
        _ => Ok(Response::new().add_attribute("action", "receive_cw20_unknown")),
    }
}

/// Callback invoked when membership weights change.
/// We need to update governance votes and funds distributor weights.
///
/// Only the membership contract can call this.
pub fn weights_changed(
    ctx: &mut Context,
    msg: WeightsChangedMsg,
) -> GovernanceControllerResult<Response> {
    let membership_contract = query_membership_addr(ctx.deps.as_ref())?;
    if ctx.info.sender != membership_contract {
        return Err(Unauthorized);
    }

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

pub fn execute_msg_reply_callback(
    ctx: &mut Context,
    msg: ExecuteMsgReplyCallbackMsg,
) -> GovernanceControllerResult<Response> {
    let ics_proxy_callback = ICS_PROXY_CALLBACKS.may_load(ctx.deps.storage, msg.callback_id)?;

    match ics_proxy_callback {
        Some(ics_proxy_callback) => {
            let canonical_sender = ctx.deps.api.addr_canonicalize(ctx.info.sender.as_ref())?;

            if canonical_sender != ics_proxy_callback.proxy_addr {
                return Err(Unauthorized);
            }

            ICS_PROXY_CALLBACKS.remove(ctx.deps.storage, msg.callback_id);

            let reply = Reply {
                id: msg.callback_id as u64,
                result: SubMsgResult::Ok(SubMsgResponse {
                    events: msg.events,
                    data: msg.data,
                }),
            };

            match ics_proxy_callback.callback_type {
                IcsProxyCallbackType::InstantiateProxy {
                    deploy_treasury_msg,
                } => handle_instantiate_proxy_reply_callback(
                    ctx,
                    ics_proxy_callback.chain_id,
                    *deploy_treasury_msg,
                    reply,
                ),
                IcsProxyCallbackType::InstantiateTreasury {} => {
                    handle_instantiate_treasury_reply_callback(
                        ctx,
                        ics_proxy_callback.chain_id,
                        reply,
                    )
                }
            }
        }
        None => Err(Unauthorized),
    }
}

fn handle_instantiate_proxy_reply_callback(
    ctx: &mut Context,
    chain_id: String,
    deploy_treasury_msg: DeployCrossChainTreasuryMsg,
    reply: Reply,
) -> GovernanceControllerResult<Response> {
    let contract_address = parse_reply_instantiate_data(reply)?.contract_address;

    let proxy_addr = ctx.deps.api.addr_canonicalize(&contract_address)?;

    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

    let add_proxy_submsg = SubMsg::new(wasm_execute(
        enterprise_contract.to_string(),
        &AddCrossChainProxy(AddCrossChainProxyMsg {
            chain_id,
            proxy_addr: proxy_addr.to_string(),
        }),
        vec![],
    )?);

    let instantiate_treasury_submsg = instantiate_remote_treasury(
        ctx.deps.branch(),
        ctx.env.clone(),
        deploy_treasury_msg.enterprise_treasury_code_id,
        proxy_addr.to_string(),
        deploy_treasury_msg.asset_whitelist,
        deploy_treasury_msg.nft_whitelist,
        deploy_treasury_msg.cross_chain_msg_spec,
    )?;

    Ok(execute_execute_msg_reply_callback_response()
        .add_submessage(add_proxy_submsg)
        .add_submessage(instantiate_treasury_submsg))
}

fn handle_instantiate_treasury_reply_callback(
    ctx: &mut Context,
    chain_id: String,
    reply: Reply,
) -> GovernanceControllerResult<Response> {
    let contract_address = parse_reply_instantiate_data(reply)?.contract_address;

    let treasury_addr = ctx.deps.api.addr_canonicalize(&contract_address)?;

    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

    let add_treasury_submsg = SubMsg::new(wasm_execute(
        enterprise_contract.to_string(),
        &AddCrossChainTreasury(AddCrossChainTreasuryMsg {
            chain_id,
            treasury_addr: treasury_addr.to_string(),
        }),
        vec![],
    )?);

    Ok(execute_execute_msg_reply_callback_response().add_submessage(add_treasury_submsg))
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
        QueryMsg::Config {} => to_binary(&query_config(qctx)?)?,
        QueryMsg::GovConfig {} => to_binary(&query_gov_config(qctx)?)?,
        QueryMsg::Proposal(params) => to_binary(&query_proposal(qctx, params)?)?,
        QueryMsg::Proposals(params) => to_binary(&query_proposals(qctx, params)?)?,
        QueryMsg::ProposalStatus(params) => to_binary(&query_proposal_status(qctx, params)?)?,
        QueryMsg::MemberVote(params) => to_binary(&query_member_vote(qctx, params)?)?,
        QueryMsg::ProposalVotes(params) => to_binary(&query_proposal_votes(qctx, params)?)?,
    };
    Ok(response)
}

pub fn query_config(qctx: QueryContext) -> GovernanceControllerResult<ConfigResponse> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(qctx.deps.storage)?;

    Ok(ConfigResponse {
        enterprise_contract,
    })
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

fn query_enterprise_treasury_addr(deps: Deps) -> GovernanceControllerResult<Addr> {
    Ok(query_enterprise_components(deps)?.enterprise_treasury_contract)
}

fn query_enterprise_cross_chain_deployments(
    deps: Deps,
    chain_id: String,
) -> GovernanceControllerResult<CrossChainDeploymentsResponse> {
    let enterprise = ENTERPRISE_CONTRACT.load(deps.storage)?;

    Ok(deps.querier.query_wasm_smart(
        enterprise.to_string(),
        &CrossChainDeployments(CrossChainDeploymentsParams { chain_id }),
    )?)
}

fn query_membership_addr(deps: Deps) -> GovernanceControllerResult<Addr> {
    Ok(query_enterprise_components(deps)?.membership_contract)
}

/// Query the membership contract for its TokenConfig.
/// Will fail if the DAO is not of type Token.
fn query_dao_token_config(deps: Deps) -> GovernanceControllerResult<TokenConfigResponse> {
    let membership_contract = query_membership_addr(deps)?;

    let token_config: TokenConfigResponse = deps
        .querier
        .query_wasm_smart(membership_contract.to_string(), &TokenConfig {})?;

    Ok(token_config)
}

/// Query the membership contract for its NftConfig.
/// Will fail if the DAO is not of type Nft.
fn query_dao_nft_config(deps: Deps) -> GovernanceControllerResult<NftConfigResponse> {
    let membership_contract = query_membership_addr(deps)?;

    let nft_config: NftConfigResponse = deps
        .querier
        .query_wasm_smart(membership_contract.to_string(), &NftConfig {})?;

    Ok(nft_config)
}

/// Query the membership contract for its DenomConfig.
/// Will fail if the DAO is not of type Denom.
fn query_dao_denom_config(deps: Deps) -> GovernanceControllerResult<DenomConfigResponse> {
    let membership_contract = query_membership_addr(deps)?;

    let denom_config: DenomConfigResponse = deps
        .querier
        .query_wasm_smart(membership_contract.to_string(), &DenomConfig {})?;

    Ok(denom_config)
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

    let total_weight: TotalWeightResponse = deps.querier.query_wasm_smart(
        dao_council_membership.to_string(),
        &multisig_membership_api::msg::QueryMsg::TotalWeight(TotalWeightParams { expiration }),
    )?;

    Ok(total_weight.total_weight)
}

/// Checks whether the user should be restricted from participating, i.e. there is an attestation
/// that they didn't sign.
fn is_restricted_user(deps: Deps, user: String) -> GovernanceControllerResult<bool> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(deps.storage)?;

    let is_restricted_user: IsRestrictedUserResponse = deps.querier.query_wasm_smart(
        enterprise_contract.to_string(),
        &IsRestrictedUser(IsRestrictedUserParams { user }),
    )?;

    Ok(is_restricted_user.is_restricted)
}

fn unrestricted_users_only(deps: Deps, user: String) -> GovernanceControllerResult<()> {
    if is_restricted_user(deps, user)? {
        return Err(RestrictedUser);
    }

    Ok(())
}
