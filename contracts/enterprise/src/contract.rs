use crate::multisig::{
    load_total_multisig_weight, load_total_multisig_weight_at_height,
    load_total_multisig_weight_at_time, save_total_multisig_weight, MULTISIG_MEMBERS,
};
use crate::nft_staking;
use crate::nft_staking::{load_all_nft_stakes_for_user, save_nft_stake, NftStake};
use crate::proposals::ProposalType::{Council, General};
use crate::proposals::{
    get_proposal_actions, is_proposal_executed, set_proposal_executed, ProposalInfo, ProposalType,
    COUNCIL_PROPOSAL_INFOS, PROPOSAL_INFOS, TOTAL_DEPOSITS,
};
use crate::staking::{
    load_total_staked, load_total_staked_at_height, load_total_staked_at_time, save_total_staked,
    CW20_STAKES,
};
use crate::state::{
    add_claim, State, ASSET_WHITELIST, CLAIMS, DAO_CODE_VERSION, DAO_COUNCIL, DAO_CREATION_DATE,
    DAO_GOV_CONFIG, DAO_MEMBERSHIP_CONTRACT, DAO_METADATA, DAO_TYPE, ENTERPRISE_FACTORY_CONTRACT,
    NFT_WHITELIST, STATE,
};
use crate::validate::{
    validate_dao_council, validate_dao_gov_config, validate_deposit,
    validate_existing_dao_contract, validate_modify_multisig_membership, validate_proposal_actions,
};
use common::cw::{Context, Pagination, QueryContext};
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{
    entry_point, from_binary, to_binary, wasm_execute, wasm_instantiate, Addr, Binary, BlockInfo,
    CosmosMsg, Decimal, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response, StdError,
    StdResult, Storage, SubMsg, Timestamp, Uint128, Uint64, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20ReceiveMsg, Logo, MinterResponse};
use cw20_base::msg::InstantiateMarketingInfo;
use cw3::VoterListResponse;
use cw721::{Cw721ReceiveMsg, TokensResponse};
use cw_asset::{Asset, AssetInfo};
use cw_storage_plus::Bound;
use cw_utils::{parse_reply_instantiate_data, Duration, Expiration};
use enterprise_factory_api::msg::QueryMsg::{GlobalAssetWhitelist, GlobalNftWhitelist};
use enterprise_protocol::api::ClaimAsset::{Cw20, Cw721};
use enterprise_protocol::api::DaoType::{Multisig, Nft};
use enterprise_protocol::api::ModifyValue::Change;
use enterprise_protocol::api::{
    AssetTreasuryResponse, AssetWhitelistResponse, CastVoteMsg, Claim, ClaimsParams,
    ClaimsResponse, CreateProposalMsg, Cw20ClaimAsset, Cw721ClaimAsset, DaoGovConfig,
    DaoInfoResponse, DaoMembershipInfo, DaoType, ExecuteMsgsMsg, ExecuteProposalMsg,
    ExistingDaoMembershipMsg, ListMultisigMembersMsg, MemberInfoResponse, MemberVoteParams,
    MemberVoteResponse, ModifyMultisigMembershipMsg, MultisigMember, MultisigMembersResponse,
    NewDaoMembershipMsg, NewMembershipInfo, NewMultisigMembershipInfo, NewNftMembershipInfo,
    NewTokenMembershipInfo, NftCollection, NftTreasuryResponse, NftUserStake, NftWhitelistResponse,
    Proposal, ProposalAction, ProposalActionType, ProposalDeposit, ProposalId, ProposalParams,
    ProposalResponse, ProposalStatus, ProposalStatusFilter, ProposalStatusParams,
    ProposalStatusResponse, ProposalVotesParams, ProposalVotesResponse, ProposalsParams,
    ProposalsResponse, QueryMemberInfoMsg, ReleaseAt, RequestFundingFromDaoMsg, TokenUserStake,
    TotalStakedAmountResponse, UnstakeMsg, UpdateAssetWhitelistMsg, UpdateCouncilMsg,
    UpdateGovConfigMsg, UpdateMetadataMsg, UpdateNftWhitelistMsg, UpgradeDaoMsg, UserStake,
    UserStakeParams, UserStakeResponse,
};
use enterprise_protocol::error::DaoError::{
    InsufficientStakedAssets, InvalidArgument, NoVotesAvailable, NotMultisigMember,
    ProposalAlreadyExecuted, UnsupportedCouncilProposalAction,
};
use enterprise_protocol::error::{DaoError, DaoResult};
use enterprise_protocol::msg::{
    Cw20HookMsg, Cw721HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use itertools::Itertools;
use nft_staking::NFT_STAKES;
use poll_engine::api::VoteOutcome::Default;
use poll_engine::api::{
    CastVoteParams, CreatePollParams, DefaultVoteOption, EndPollParams, PollId, PollParams,
    PollStatus, PollStatusFilter, PollType, PollVoterParams, PollVotersParams, PollsParams,
    VotingScheme,
};
use poll_engine::error::PollError;
use poll_engine::error::PollError::PollInProgress;
use poll_engine::execute::{create_poll, end_poll, initialize_poll_engine};
use poll_engine::query::{
    query_poll, query_poll_status, query_poll_voter, query_poll_voters, query_polls, query_voter,
};
use poll_engine::state::Poll;
use std::cmp::min;
use std::ops::{Add, Not, Sub};
use DaoError::{
    CustomError, NoDaoCouncil, NoNftTokenStaked, NoSuchProposal, Unauthorized,
    UnsupportedOperationForDaoType, ZeroInitialWeightMember,
};
use DaoMembershipInfo::{Existing, New};
use DaoType::Token;
use NewMembershipInfo::{NewMultisig, NewNft, NewToken};
use ProposalAction::{
    ExecuteMsgs, ModifyMultisigMembership, RequestFundingFromDao, UpdateAssetWhitelist,
    UpdateCouncil, UpdateGovConfig, UpdateMetadata, UpdateNftWhitelist, UpgradeDao,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DAO_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 1;

pub const CODE_VERSION: u64 = 1;

pub const PROPOSAL_OUTCOME_YES: u8 = 0;
pub const PROPOSAL_OUTCOME_NO: u8 = 1;
pub const PROPOSAL_OUTCOME_ABSTAIN: u8 = 2;
pub const PROPOSAL_OUTCOME_VETO: u8 = 3;

pub const DEFAULT_QUERY_LIMIT: u32 = 50u32;
pub const MAX_QUERY_LIMIT: u32 = 100u32;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> DaoResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    validate_dao_gov_config(
        &dao_type_from_membership(&msg.dao_membership_info),
        &msg.dao_gov_config,
    )?;

    validate_dao_council(msg.dao_council.clone())?;

    STATE.save(
        deps.storage,
        &State {
            multisig_dao_proposal_actions: None,
        },
    )?;

    DAO_CREATION_DATE.save(deps.storage, &env.block.time)?;

    DAO_METADATA.save(deps.storage, &msg.dao_metadata)?;
    DAO_GOV_CONFIG.save(deps.storage, &msg.dao_gov_config)?;
    DAO_COUNCIL.save(deps.storage, &msg.dao_council)?;
    ENTERPRISE_FACTORY_CONTRACT.save(
        deps.storage,
        &deps.api.addr_validate(&msg.enterprise_factory_contract)?,
    )?;
    DAO_CODE_VERSION.save(deps.storage, &CODE_VERSION.into())?;

    ASSET_WHITELIST.save(deps.storage, &msg.asset_whitelist.unwrap_or_default())?;
    for nft in &msg.nft_whitelist.unwrap_or_default() {
        NFT_WHITELIST.save(deps.storage, nft.clone(), &())?;
    }
    save_total_staked(deps.storage, &Uint128::zero(), &env.block)?;
    TOTAL_DEPOSITS.save(deps.storage, &Uint128::zero())?;

    let mut ctx = Context { deps, env, info };
    initialize_poll_engine(&mut ctx)?;

    let submessages = match msg.dao_membership_info {
        New(membership) => instantiate_new_membership_dao(ctx, membership)?,
        Existing(membership) => instantiate_existing_membership_dao(ctx, membership)?,
    };

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_submessages(submessages))
}

fn dao_type_from_membership(membership_info: &DaoMembershipInfo) -> DaoType {
    match membership_info {
        New(info) => match info.membership_info {
            NewToken(_) => Token,
            NewNft(_) => Nft,
            NewMultisig(_) => Multisig,
        },
        Existing(info) => info.dao_type.clone(),
    }
}

fn instantiate_new_membership_dao(
    ctx: Context,
    membership: NewDaoMembershipMsg,
) -> DaoResult<Vec<SubMsg>> {
    match membership.membership_info {
        NewToken(info) => {
            instantiate_new_token_dao(ctx, info, membership.membership_contract_code_id)
        }
        NewNft(info) => instantiate_new_nft_dao(ctx, info, membership.membership_contract_code_id),
        NewMultisig(info) => instantiate_new_multisig_dao(ctx, info),
    }
}

fn instantiate_new_token_dao(
    ctx: Context,
    info: NewTokenMembershipInfo,
    cw20_code_id: u64,
) -> DaoResult<Vec<SubMsg>> {
    for initial_balance in info.initial_token_balances.iter() {
        if initial_balance.amount == Uint128::zero() {
            return Err(ZeroInitialWeightMember);
        }
    }

    DAO_TYPE.save(ctx.deps.storage, &Token)?;

    let marketing = info
        .token_marketing
        .map(|marketing| InstantiateMarketingInfo {
            project: marketing.project,
            description: marketing.description,
            marketing: marketing.marketing_owner,
            logo: marketing.logo_url.map(Logo::Url),
        });

    let create_token_msg = cw20_base::msg::InstantiateMsg {
        name: info.token_name.clone(),
        symbol: info.token_symbol,
        decimals: info.token_decimals,
        initial_balances: info.initial_token_balances,
        mint: info.token_mint.or_else(|| {
            Some(MinterResponse {
                minter: ctx.env.contract.address.to_string(),
                cap: None,
            })
        }),
        marketing,
    };
    let submsg = SubMsg::reply_on_success(
        wasm_instantiate(cw20_code_id, &create_token_msg, vec![], info.token_name)?,
        DAO_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
    );
    Ok(vec![submsg])
}

fn instantiate_new_multisig_dao(
    ctx: Context,
    info: NewMultisigMembershipInfo,
) -> DaoResult<Vec<SubMsg>> {
    DAO_TYPE.save(ctx.deps.storage, &Multisig)?;

    let mut total_weight = Uint128::zero();

    for member in info.multisig_members.iter() {
        if member.weight == Uint128::zero() {
            return Err(ZeroInitialWeightMember);
        }

        let member_addr = ctx.deps.api.addr_validate(&member.address)?;
        MULTISIG_MEMBERS.save(ctx.deps.storage, member_addr, &member.weight)?;

        total_weight = total_weight.add(member.weight);
    }

    save_total_multisig_weight(ctx.deps.storage, total_weight, &ctx.env.block)?;

    // TODO: do we even need this? just for consistency, but otherwise makes no sense since multisig members are now internally stored
    DAO_MEMBERSHIP_CONTRACT.save(ctx.deps.storage, &ctx.env.contract.address)?;

    Ok(vec![])
}

fn instantiate_new_nft_dao(
    ctx: Context,
    info: NewNftMembershipInfo,
    cw721_code_id: u64,
) -> DaoResult<Vec<SubMsg>> {
    DAO_TYPE.save(ctx.deps.storage, &Nft)?;

    let minter = match info.minter {
        None => ctx.env.contract.address.to_string(),
        Some(minter) => minter,
    };
    let instantiate_msg = cw721_base::InstantiateMsg {
        name: info.nft_name,
        symbol: info.nft_symbol,
        minter,
    };
    let submsg = SubMsg::reply_on_success(
        wasm_instantiate(
            cw721_code_id,
            &instantiate_msg,
            vec![],
            "DAO NFT".to_string(),
        )?,
        DAO_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
    );
    Ok(vec![submsg])
}

fn instantiate_existing_membership_dao(
    ctx: Context,
    membership: ExistingDaoMembershipMsg,
) -> DaoResult<Vec<SubMsg>> {
    let membership_addr = ctx
        .deps
        .api
        .addr_validate(&membership.membership_contract_addr)?;

    validate_existing_dao_contract(
        &ctx,
        &membership.dao_type,
        &membership.membership_contract_addr,
    )?;

    DAO_TYPE.save(ctx.deps.storage, &membership.dao_type)?;

    match membership.dao_type {
        Token | Nft => {
            DAO_MEMBERSHIP_CONTRACT.save(ctx.deps.storage, &membership_addr)?;
        }
        Multisig => {
            DAO_MEMBERSHIP_CONTRACT.save(ctx.deps.storage, &ctx.env.contract.address)?;

            // TODO: gotta do an integration test for this
            let mut total_weight = Uint128::zero();
            let mut last_voter: Option<String> = None;
            while {
                let query_msg = cw3::Cw3QueryMsg::ListVoters {
                    start_after: last_voter.clone(),
                    limit: None,
                };

                last_voter = None;

                let voters: VoterListResponse = ctx
                    .deps
                    .querier
                    .query_wasm_smart(&membership.membership_contract_addr, &query_msg)?;

                for voter in voters.voters {
                    last_voter = Some(voter.addr.clone());

                    let voter_addr = ctx.deps.api.addr_validate(&voter.addr)?;
                    MULTISIG_MEMBERS.save(ctx.deps.storage, voter_addr, &voter.weight.into())?;

                    total_weight = total_weight.add(Uint128::from(voter.weight));
                }

                last_voter.is_some()
            } {}

            save_total_multisig_weight(ctx.deps.storage, total_weight, &ctx.env.block)?;
        }
    }

    Ok(vec![])
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> DaoResult<Response> {
    let mut ctx = Context { deps, env, info };
    match msg {
        ExecuteMsg::CreateProposal(msg) => create_proposal(&mut ctx, msg, None),
        ExecuteMsg::CreateCouncilProposal(msg) => create_council_proposal(&mut ctx, msg),
        ExecuteMsg::CastVote(msg) => cast_vote(&mut ctx, msg),
        ExecuteMsg::CastCouncilVote(msg) => cast_council_vote(&mut ctx, msg),
        ExecuteMsg::ExecuteProposal(msg) => execute_proposal(&mut ctx, msg, General),
        ExecuteMsg::ExecuteCouncilProposal(msg) => execute_proposal(&mut ctx, msg, Council),
        ExecuteMsg::Receive(msg) => receive_cw20(&mut ctx, msg),
        ExecuteMsg::ReceiveNft(msg) => receive_cw721(&mut ctx, msg),
        ExecuteMsg::Unstake(msg) => unstake(&mut ctx, msg),
        ExecuteMsg::Claim {} => claim(&mut ctx),
    }
}

fn create_proposal(
    ctx: &mut Context,
    msg: CreateProposalMsg,
    deposit: Option<ProposalDeposit>,
) -> DaoResult<Response> {
    let gov_config = DAO_GOV_CONFIG.load(ctx.deps.storage)?;

    validate_deposit(&gov_config, &deposit)?;
    validate_proposal_actions(ctx.deps.as_ref(), &msg.proposal_actions)?;

    let ends_at = ctx.env.block.time.plus_seconds(gov_config.vote_duration);

    let dao_type = DAO_TYPE.load(ctx.deps.storage)?;

    let base_response = Response::new()
        .add_attribute("action", "create_proposal")
        .add_attribute("dao_address", ctx.env.contract.address.to_string());

    match dao_type {
        Token => {
            let poll_id =
                create_poll_engine_proposal(ctx, gov_config, ends_at, msg, deposit, General)?;

            Ok(base_response.add_attribute("proposal_id", poll_id.to_string()))
        }
        Nft => {
            if !user_has_nfts_staked(ctx)? && !user_holds_nft(ctx)? {
                return Err(DaoError::NotNftOwner {});
            }

            let poll_id =
                create_poll_engine_proposal(ctx, gov_config, ends_at, msg, deposit, General)?;

            Ok(base_response.add_attribute("proposal_id", poll_id.to_string()))
        }
        Multisig => {
            let member_weight = MULTISIG_MEMBERS
                .may_load(ctx.deps.storage, ctx.info.sender.clone())?
                .unwrap_or_default();
            if member_weight == Uint128::zero() {
                return Err(NotMultisigMember {});
            }

            let poll_id =
                create_poll_engine_proposal(ctx, gov_config, ends_at, msg, deposit, General)?;

            Ok(base_response.add_attribute("proposal_id", poll_id.to_string()))
        }
    }
}

fn user_has_nfts_staked(ctx: &Context) -> StdResult<bool> {
    Ok(NFT_STAKES()
        .idx
        .staker
        .prefix(ctx.info.sender.clone())
        .range(ctx.deps.storage, None, None, Ascending)
        .next()
        .transpose()?
        .is_some())
}

fn user_holds_nft(ctx: &Context) -> StdResult<bool> {
    let cw721_contract = DAO_MEMBERSHIP_CONTRACT.load(ctx.deps.storage)?;

    let query_tokens_msg = cw721::Cw721QueryMsg::Tokens {
        owner: ctx.info.sender.to_string(),
        start_after: None,
        limit: Some(1u32),
    };
    let tokens: TokensResponse = ctx
        .deps
        .querier
        .query_wasm_smart(cw721_contract.to_string(), &query_tokens_msg)?;

    Ok(tokens.tokens.is_empty().not())
}

// TODO: tests
fn create_council_proposal(ctx: &mut Context, msg: CreateProposalMsg) -> DaoResult<Response> {
    let dao_council = DAO_COUNCIL.load(ctx.deps.storage)?;

    match dao_council {
        None => Err(NoDaoCouncil),
        Some(dao_council) => {
            let proposer = ctx.info.sender.to_string();

            if !dao_council.members.contains(&proposer) {
                return Err(Unauthorized);
            }

            let allowed_actions = dao_council
                .allowed_proposal_action_types
                .unwrap_or_else(|| vec![ProposalActionType::UpgradeDao]);

            // TODO: ban certain action types like funding and modify membership? they are unsafe, the council could hijack the DAO

            // validate that proposal actions are allowed
            for proposal_action in &msg.proposal_actions {
                let proposal_action_type = to_proposal_action_type(proposal_action);
                if !allowed_actions.contains(&proposal_action_type) {
                    return Err(UnsupportedCouncilProposalAction {
                        action: proposal_action_type,
                    });
                }
            }

            let gov_config = DAO_GOV_CONFIG.load(ctx.deps.storage)?;

            let council_gov_config = DaoGovConfig {
                quorum: Decimal::percent(75u64),
                threshold: Decimal::percent(50u64),
                vote_duration: u64::MAX, // TODO: probably not going to work
                ..gov_config
            };

            // TODO: this ends_at is not good probably, won't allow ending of the proposal even when targets are reached
            let proposal_id = create_poll_engine_proposal(
                ctx,
                council_gov_config,
                Timestamp::from_seconds(u64::MAX),
                msg,
                None,
                General,
            )?;

            Ok(Response::new()
                .add_attribute("action", "create_council_proposal")
                .add_attribute("proposal_id", proposal_id.to_string()))
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
    }
}

fn create_poll_engine_proposal(
    ctx: &mut Context,
    gov_config: DaoGovConfig,
    ends_at: Timestamp,
    msg: CreateProposalMsg,
    deposit: Option<ProposalDeposit>,
    proposal_type: ProposalType,
) -> DaoResult<PollId> {
    let poll = create_poll(
        ctx,
        CreatePollParams {
            proposer: ctx.info.sender.to_string(),
            deposit_amount: Uint128::zero(),
            label: msg.title,
            description: msg.description.unwrap_or_default(),
            poll_type: PollType::Multichoice {
                threshold: gov_config.threshold,
                n_outcomes: 4,
                rejecting_outcomes: vec![PROPOSAL_OUTCOME_NO, PROPOSAL_OUTCOME_VETO],
            },
            scheme: VotingScheme::CoinVoting,
            ends_at,
            quorum: gov_config.quorum,
        },
    )?;

    match proposal_type {
        General => PROPOSAL_INFOS.save(
            ctx.deps.storage,
            poll.id,
            &ProposalInfo {
                executed_at: None,
                proposal_deposit: deposit.clone(),
                proposal_actions: msg.proposal_actions,
            },
        )?,
        Council => COUNCIL_PROPOSAL_INFOS.save(
            ctx.deps.storage,
            poll.id,
            &ProposalInfo {
                executed_at: None,
                proposal_deposit: deposit.clone(),
                proposal_actions: msg.proposal_actions,
            },
        )?,
    }

    if let Some(deposit) = deposit {
        TOTAL_DEPOSITS.update(ctx.deps.storage, |deposits| -> StdResult<Uint128> {
            Ok(deposits.add(deposit.amount))
        })?;
    }

    Ok(poll.id)
}

fn cast_vote(ctx: &mut Context, msg: CastVoteMsg) -> DaoResult<Response> {
    let qctx = QueryContext::from(ctx.deps.as_ref(), ctx.env.clone());
    let user_available_votes = get_user_available_votes(qctx, ctx.info.sender.clone())?;

    if user_available_votes == Uint128::zero() {
        return Err(Unauthorized);
    }

    poll_engine::execute::cast_vote(
        ctx,
        CastVoteParams {
            poll_id: msg.proposal_id.into(),
            outcome: Default(msg.outcome),
            amount: user_available_votes,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "cast_vote")
        .add_attribute("dao_address", ctx.env.contract.address.to_string())
        .add_attribute("proposal_id", msg.proposal_id.to_string())
        .add_attribute("voter", ctx.info.sender.clone().to_string())
        .add_attribute("outcome", msg.outcome.to_string())
        .add_attribute("amount", user_available_votes.to_string()))
}

// TODO: test
fn cast_council_vote(ctx: &mut Context, msg: CastVoteMsg) -> DaoResult<Response> {
    let dao_council = DAO_COUNCIL.load(ctx.deps.storage)?;

    match dao_council {
        None => Err(NoDaoCouncil),
        Some(dao_council) => {
            if !dao_council.members.contains(&ctx.info.sender.to_string()) {
                return Err(Unauthorized);
            }

            // ensure this proposal is actually a council proposal
            if !COUNCIL_PROPOSAL_INFOS.has(ctx.deps.storage, msg.proposal_id) {
                return Err(NoSuchProposal);
            }

            poll_engine::execute::cast_vote(
                ctx,
                CastVoteParams {
                    poll_id: msg.proposal_id.into(),
                    outcome: Default(msg.outcome),
                    amount: Uint128::one(),
                },
            )?;

            Ok(Response::new()
                .add_attribute("action", "cast_vote")
                .add_attribute("dao_address", ctx.env.contract.address.to_string())
                .add_attribute("proposal_id", msg.proposal_id.to_string())
                .add_attribute("voter", ctx.info.sender.clone().to_string())
                .add_attribute("outcome", msg.outcome.to_string())
                .add_attribute("amount", 1u8.to_string()))
        }
    }
}

// TODO: think whether this should be renamed to 'end_proposal'
// TODO: test for proposal type council
fn execute_proposal(
    ctx: &mut Context,
    msg: ExecuteProposalMsg,
    proposal_type: ProposalType,
) -> DaoResult<Response> {
    if is_proposal_executed(ctx.deps.storage, msg.proposal_id, proposal_type.clone())? {
        return Err(ProposalAlreadyExecuted);
    }

    let submsgs = execute_poll_engine_proposal(ctx, &msg, proposal_type)?;

    Ok(Response::new()
        .add_submessages(submsgs)
        .add_attribute("action", "execute_proposal")
        .add_attribute("dao_address", ctx.env.contract.address.to_string())
        .add_attribute("proposal_id", msg.proposal_id.to_string()))
}

fn return_proposal_deposit_submsgs(
    ctx: &mut Context,
    proposal_id: ProposalId,
) -> DaoResult<Vec<SubMsg>> {
    let proposal_info = PROPOSAL_INFOS
        .may_load(ctx.deps.storage, proposal_id)?
        .ok_or(NoSuchProposal)?;

    return_deposit_submsgs(ctx, proposal_info.proposal_deposit)
}

fn return_deposit_submsgs(
    ctx: &mut Context,
    deposit: Option<ProposalDeposit>,
) -> DaoResult<Vec<SubMsg>> {
    match deposit {
        None => Ok(vec![]),
        Some(deposit) => {
            let membership_contract = DAO_MEMBERSHIP_CONTRACT.load(ctx.deps.storage)?;

            let transfer_msg =
                Asset::cw20(membership_contract, deposit.amount).transfer_msg(deposit.depositor)?;

            TOTAL_DEPOSITS.update(ctx.deps.storage, |deposits| -> StdResult<Uint128> {
                Ok(deposits.sub(deposit.amount))
            })?;

            Ok(vec![SubMsg::new(transfer_msg)])
        }
    }
}

fn execute_poll_engine_proposal(
    ctx: &mut Context,
    msg: &ExecuteProposalMsg,
    proposal_type: ProposalType,
) -> DaoResult<Vec<SubMsg>> {
    let qctx = QueryContext::from(ctx.deps.as_ref(), ctx.env.clone());
    let poll = query_poll(
        &qctx,
        PollParams {
            poll_id: msg.proposal_id,
        },
    )?;

    let total_available_votes = match proposal_type {
        General => {
            let dao_type = DAO_TYPE.load(ctx.deps.storage)?;
            match dao_type {
                Token | Nft => {
                    if ctx.env.block.time >= poll.poll.ends_at {
                        load_total_staked_at_time(ctx.deps.storage, poll.poll.ends_at)?
                    } else {
                        load_total_staked(ctx.deps.storage)?
                    }
                }
                Multisig => {
                    // TODO: add tests for this
                    if ctx.env.block.time >= poll.poll.ends_at {
                        load_total_multisig_weight_at_time(ctx.deps.storage, poll.poll.ends_at)?
                    } else {
                        load_total_multisig_weight(ctx.deps.storage)?
                    }
                }
            }
        }
        Council => {
            let dao_council = DAO_COUNCIL.load(ctx.deps.storage)?;

            match dao_council {
                None => return Err(NoDaoCouncil),
                Some(dao_council) => Uint128::from(dao_council.members.len() as u128),
            }
        }
    };

    if total_available_votes == Uint128::zero() {
        return Err(NoVotesAvailable);
    }

    end_poll(
        ctx,
        EndPollParams {
            poll_id: msg.proposal_id.into(),
            maximum_available_votes: total_available_votes,
            error_if_already_ended: false,
        },
    )?;

    let qctx = QueryContext::from(ctx.deps.as_ref(), ctx.env.clone());
    let poll_status = query_poll_status(&qctx, msg.proposal_id)?.status;

    let submsgs = match poll_status {
        PollStatus::InProgress { .. } => {
            return Err(PollInProgress {
                poll_id: msg.proposal_id.into(),
            }
            .into())
        }
        PollStatus::Passed { .. } => {
            set_proposal_executed(
                ctx.deps.storage,
                msg.proposal_id,
                ctx.env.block.clone(),
                proposal_type.clone(),
            )?;
            let mut submsgs = execute_proposal_actions(ctx, msg.proposal_id, proposal_type)?;
            let mut return_deposit = return_proposal_deposit_submsgs(ctx, msg.proposal_id)?;
            submsgs.append(&mut return_deposit);

            submsgs
        }
        PollStatus::Rejected { outcome, .. } => {
            set_proposal_executed(
                ctx.deps.storage,
                msg.proposal_id,
                ctx.env.block.clone(),
                proposal_type,
            )?;

            // return deposits only if not vetoed
            if Some(PROPOSAL_OUTCOME_VETO) != outcome {
                return_proposal_deposit_submsgs(ctx, msg.proposal_id)?
            } else {
                let proposal_deposit = PROPOSAL_INFOS
                    .may_load(ctx.deps.storage, msg.proposal_id)?
                    .ok_or(NoSuchProposal)?
                    .proposal_deposit;
                if let Some(deposit) = proposal_deposit {
                    TOTAL_DEPOSITS.update(ctx.deps.storage, |deposits| -> StdResult<Uint128> {
                        Ok(deposits.sub(deposit.amount))
                    })?;
                }
                vec![]
            }
        }
    };

    Ok(submsgs)
}

fn execute_proposal_actions(
    ctx: &mut Context,
    proposal_id: ProposalId,
    proposal_type: ProposalType,
) -> DaoResult<Vec<SubMsg>> {
    let proposal_actions = get_proposal_actions(ctx.deps.storage, proposal_id, proposal_type)?
        .ok_or(NoSuchProposal)?;

    let mut submsgs: Vec<SubMsg> = vec![];

    for proposal_action in proposal_actions {
        // TODO: avoid this '.append(&mut ...)' stuff somehow, really ugly
        submsgs.append(&mut match proposal_action {
            UpdateMetadata(msg) => update_metadata(ctx, msg)?,
            UpdateGovConfig(msg) => update_gov_config(ctx, msg)?,
            UpdateCouncil(msg) => update_council(ctx, msg)?,
            RequestFundingFromDao(msg) => execute_funding_from_dao(msg)?,
            UpdateAssetWhitelist(msg) => update_asset_whitelist(ctx, msg)?,
            UpdateNftWhitelist(msg) => update_nft_whitelist(ctx, msg)?,
            UpgradeDao(msg) => upgrade_dao(ctx, msg)?,
            ExecuteMsgs(msg) => execute_msgs(ctx, msg)?,
            ModifyMultisigMembership(msg) => modify_multisig_membership(ctx, msg)?,
        })
    }

    Ok(submsgs) // TODO: maybe also include attributes from various actions this took?
}

fn update_metadata(ctx: &mut Context, msg: UpdateMetadataMsg) -> DaoResult<Vec<SubMsg>> {
    let mut metadata = DAO_METADATA.load(ctx.deps.storage)?;

    if let Change(name) = msg.name {
        metadata.name = name;
    }

    if let Change(logo) = msg.logo {
        metadata.logo = logo;
    }

    if let Change(github) = msg.github_username {
        metadata.socials.github_username = github;
    }
    if let Change(twitter) = msg.twitter_username {
        metadata.socials.twitter_username = twitter;
    }
    if let Change(discord) = msg.discord_username {
        metadata.socials.discord_username = discord;
    }
    if let Change(telegram) = msg.telegram_username {
        metadata.socials.telegram_username = telegram;
    }

    DAO_METADATA.save(ctx.deps.storage, &metadata)?;

    Ok(vec![])
}

fn execute_funding_from_dao(msg: RequestFundingFromDaoMsg) -> DaoResult<Vec<SubMsg>> {
    // TODO: does not work with CW1155, make sure it does in the future
    let submsgs = msg
        .assets
        .into_iter()
        .map(|asset| asset.transfer_msg(msg.recipient.clone()))
        .collect::<StdResult<Vec<CosmosMsg>>>()?
        .into_iter()
        .map(SubMsg::new)
        .collect_vec();

    Ok(submsgs)
}

fn update_gov_config(ctx: &mut Context, msg: UpdateGovConfigMsg) -> DaoResult<Vec<SubMsg>> {
    let mut gov_config = DAO_GOV_CONFIG.load(ctx.deps.storage)?;

    if let Change(quorum) = msg.quorum {
        gov_config.quorum = quorum;
    }

    if let Change(threshold) = msg.threshold {
        gov_config.threshold = threshold;
    }

    if let Change(voting_duration) = msg.voting_duration {
        gov_config.vote_duration = voting_duration.u64();
    }

    if let Change(unlocking_period) = msg.unlocking_period {
        gov_config.unlocking_period = unlocking_period;
    }

    if let Change(minimum_deposit) = msg.minimum_deposit {
        gov_config.minimum_deposit = minimum_deposit;
    }

    DAO_GOV_CONFIG.save(ctx.deps.storage, &gov_config)?;

    Ok(vec![])
}

// TODO: test
fn update_council(ctx: &mut Context, msg: UpdateCouncilMsg) -> DaoResult<Vec<SubMsg>> {
    DAO_COUNCIL.save(ctx.deps.storage, &msg.dao_council)?;

    Ok(vec![])
}

fn update_asset_whitelist(
    ctx: &mut Context,
    msg: UpdateAssetWhitelistMsg,
) -> DaoResult<Vec<SubMsg>> {
    let mut asset_whitelist = ASSET_WHITELIST.load(ctx.deps.storage)?;

    for add in msg.add {
        asset_whitelist.push(add);
    }

    asset_whitelist = asset_whitelist
        .into_iter()
        .filter(|asset| !msg.remove.contains(asset))
        .collect_vec();

    ASSET_WHITELIST.save(ctx.deps.storage, &asset_whitelist)?;

    Ok(vec![])
}

fn update_nft_whitelist(ctx: &mut Context, msg: UpdateNftWhitelistMsg) -> DaoResult<Vec<SubMsg>> {
    for add in msg.add {
        NFT_WHITELIST.save(ctx.deps.storage, add, &())?;
    }
    for remove in msg.remove {
        NFT_WHITELIST.remove(ctx.deps.storage, remove);
    }

    Ok(vec![])
}

fn upgrade_dao(ctx: &mut Context, msg: UpgradeDaoMsg) -> DaoResult<Vec<SubMsg>> {
    Ok(vec![SubMsg::new(WasmMsg::Migrate {
        contract_addr: ctx.env.contract.address.to_string(),
        new_code_id: msg.new_dao_code_id,
        msg: msg.migrate_msg,
    })])
}

// TODO: tests
fn execute_msgs(_ctx: &mut Context, msg: ExecuteMsgsMsg) -> DaoResult<Vec<SubMsg>> {
    let mut submsgs: Vec<SubMsg> = vec![];
    for msg in msg.msgs {
        submsgs.push(SubMsg::new(
            serde_json_wasm::from_str::<CosmosMsg>(msg.as_str()).map_err(|_| InvalidArgument {
                msg: "Error parsing message into Cosmos message".to_string(),
            })?,
        ))
    }
    Ok(submsgs)
}

// TODO: tests
fn modify_multisig_membership(
    ctx: &mut Context,
    msg: ModifyMultisigMembershipMsg,
) -> DaoResult<Vec<SubMsg>> {
    validate_modify_multisig_membership(ctx.deps.as_ref(), &msg)?;

    let mut total_weight = load_total_multisig_weight(ctx.deps.storage)?;

    for edit_member in msg.edit_members {
        let member_addr = ctx.deps.api.addr_validate(&edit_member.address)?;

        let old_member_weight = MULTISIG_MEMBERS
            .may_load(ctx.deps.storage, member_addr.clone())?
            .unwrap_or_default();

        if edit_member.weight == Uint128::zero() {
            MULTISIG_MEMBERS.remove(ctx.deps.storage, member_addr.clone())
        } else {
            MULTISIG_MEMBERS.save(ctx.deps.storage, member_addr.clone(), &edit_member.weight)?
        }

        if old_member_weight != edit_member.weight {
            update_user_poll_engine_votes(ctx, member_addr)?;

            total_weight = if old_member_weight > edit_member.weight {
                total_weight.sub(old_member_weight.sub(edit_member.weight))
            } else {
                total_weight.add(edit_member.weight.sub(old_member_weight))
            }
        }
    }

    save_total_multisig_weight(ctx.deps.storage, total_weight, &ctx.env.block)?;

    Ok(vec![])
}

pub fn receive_cw20(ctx: &mut Context, cw20_msg: Cw20ReceiveMsg) -> DaoResult<Response> {
    // only membership CW20 contract can execute this message
    let dao_type = DAO_TYPE.load(ctx.deps.storage)?;
    let membership_contract = DAO_MEMBERSHIP_CONTRACT.load(ctx.deps.storage)?;
    if dao_type != Token || ctx.info.sender != membership_contract {
        return Err(DaoError::InvalidStakingAsset);
    }

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Stake {}) => {
            let total_staked = load_total_staked(ctx.deps.storage)?;
            let new_total_staked = total_staked.add(cw20_msg.amount);
            save_total_staked(ctx.deps.storage, &new_total_staked, &ctx.env.block)?;

            let sender = ctx.deps.api.addr_validate(&cw20_msg.sender)?;
            let stake = CW20_STAKES
                .may_load(ctx.deps.storage, sender.clone())?
                .unwrap_or_default();

            let new_stake = stake.add(cw20_msg.amount);
            CW20_STAKES.save(ctx.deps.storage, sender, &new_stake)?;

            Ok(Response::new()
                .add_attribute("action", "stake_cw20")
                .add_attribute("total_staked", new_total_staked.to_string())
                .add_attribute("stake", new_stake.to_string()))
        }
        Ok(Cw20HookMsg::CreateProposal(msg)) => {
            let depositor = ctx.deps.api.addr_validate(&cw20_msg.sender)?;
            let deposit = ProposalDeposit {
                depositor,
                amount: cw20_msg.amount,
            };
            create_proposal(ctx, msg, Some(deposit))
        }
        _ => Err(CustomError {
            val: "msg payload not recognized".to_string(),
        }),
    }
}

pub fn receive_cw721(ctx: &mut Context, cw721_msg: Cw721ReceiveMsg) -> DaoResult<Response> {
    // only membership CW721 contract can execute this message
    let dao_type = DAO_TYPE.load(ctx.deps.storage)?;
    let membership_contract = DAO_MEMBERSHIP_CONTRACT.load(ctx.deps.storage)?;
    if dao_type != Nft || ctx.info.sender != membership_contract {
        return Err(DaoError::InvalidStakingAsset);
    }

    match from_binary(&cw721_msg.msg) {
        Ok(Cw721HookMsg::Stake {}) => {
            let token_id = cw721_msg.token_id;

            let existing_stake = NFT_STAKES().may_load(ctx.deps.storage, token_id.clone())?;

            if existing_stake.is_some() {
                return Err(DaoError::NftTokenAlreadyStaked { token_id });
            }

            let total_staked = load_total_staked(ctx.deps.storage)?;
            let new_total_staked = total_staked.add(Uint128::from(1u8));
            save_total_staked(ctx.deps.storage, &new_total_staked, &ctx.env.block)?;

            let staker = ctx.deps.api.addr_validate(&cw721_msg.sender)?;

            let nft_stake = NftStake { staker, token_id };

            save_nft_stake(ctx.deps.storage, &nft_stake)?;

            Ok(Response::new()
                .add_attribute("action", "stake_cw721")
                .add_attribute("total_staked", new_total_staked.to_string()))
        }
        _ => Err(CustomError {
            val: "msg payload not recognized".to_string(),
        }),
    }
}

pub fn unstake(ctx: &mut Context, msg: UnstakeMsg) -> DaoResult<Response> {
    let dao_type = DAO_TYPE.load(ctx.deps.storage)?;

    match msg {
        UnstakeMsg::Cw20(msg) => {
            // TODO: test
            if dao_type != Token {
                return Err(DaoError::InvalidStakingAsset);
            }

            let stake = CW20_STAKES
                .may_load(ctx.deps.storage, ctx.info.sender.clone())?
                .unwrap_or_default();

            if stake < msg.amount {
                return Err(InsufficientStakedAssets);
            }

            let total_staked = load_total_staked(ctx.deps.storage)?;
            let new_total_staked = total_staked.sub(msg.amount);
            save_total_staked(ctx.deps.storage, &new_total_staked, &ctx.env.block)?;

            let new_stake = stake.sub(msg.amount);
            CW20_STAKES.save(ctx.deps.storage, ctx.info.sender.clone(), &new_stake)?;

            // TODO: extract to function, this is duplicate code
            let gov_config = DAO_GOV_CONFIG.load(ctx.deps.storage)?;
            let release_at = match gov_config.unlocking_period {
                Duration::Height(height) => {
                    ReleaseAt::Height((ctx.env.block.height + height).into())
                }
                Duration::Time(time) => ReleaseAt::Timestamp(ctx.env.block.time.plus_seconds(time)),
            };

            add_claim(
                ctx.deps.storage,
                &ctx.info.sender,
                Claim {
                    asset: Cw20(Cw20ClaimAsset { amount: msg.amount }),
                    release_at,
                },
            )?;

            update_user_poll_engine_votes(ctx, ctx.info.sender.clone())?;

            Ok(Response::new()
                .add_attribute("action", "unstake_cw20")
                .add_attribute("total_staked", new_total_staked.to_string())
                .add_attribute("stake", new_stake.to_string()))
        }
        UnstakeMsg::Cw721(msg) => {
            // TODO: test
            if dao_type != Nft {
                return Err(DaoError::InvalidStakingAsset);
            }

            for token in &msg.tokens {
                // TODO: might be too slow, can we load this in a batch?
                let nft_stake = NFT_STAKES().may_load(ctx.deps.storage, token.to_string())?;

                match nft_stake {
                    None => {
                        // TODO: test
                        return Err(NoNftTokenStaked {
                            token_id: token.to_string(),
                        });
                    }
                    Some(stake) => {
                        if stake.staker != ctx.info.sender {
                            // TODO: test
                            return Err(Unauthorized);
                        } else {
                            NFT_STAKES().remove(ctx.deps.storage, token.to_string())?;
                        }
                    }
                }
            }

            let total_staked = load_total_staked(ctx.deps.storage)?;
            let new_total_staked = total_staked.sub(Uint128::from(msg.tokens.len() as u128));
            save_total_staked(ctx.deps.storage, &new_total_staked, &ctx.env.block)?;

            let gov_config = DAO_GOV_CONFIG.load(ctx.deps.storage)?;
            let release_at = match gov_config.unlocking_period {
                Duration::Height(height) => {
                    // TODO: test
                    ReleaseAt::Height((ctx.env.block.height + height).into())
                }
                Duration::Time(time) => ReleaseAt::Timestamp(ctx.env.block.time.plus_seconds(time)),
            };

            add_claim(
                ctx.deps.storage,
                &ctx.info.sender,
                Claim {
                    asset: Cw721(Cw721ClaimAsset { tokens: msg.tokens }),
                    release_at,
                },
            )?;

            update_user_poll_engine_votes(ctx, ctx.info.sender.clone())?;

            Ok(Response::new()
                .add_attribute("action", "unstake_cw721")
                .add_attribute("total_staked", new_total_staked.to_string()))
        }
    }
}

pub fn update_user_poll_engine_votes(ctx: &mut Context, user: Addr) -> DaoResult<()> {
    let qctx = QueryContext::from(ctx.deps.as_ref(), ctx.env.clone());
    let votes = query_voter(&qctx, &user)?.votes;

    let user_available_votes = get_user_available_votes(qctx, user.clone())?;

    let cast_vote_ctx = &mut Context {
        deps: ctx.deps.branch(),
        env: ctx.env.clone(),
        info: MessageInfo {
            sender: user,
            funds: ctx.info.funds.clone(),
        },
    };

    for vote in votes {
        // TODO: remove this when qctx.clone() is in
        let qctx = QueryContext::from(cast_vote_ctx.deps.as_ref(), ctx.env.clone());
        let status = query_poll_status(&qctx, vote.poll_id)?;
        if let PollStatus::InProgress { ends_at } = status.status {
            if ends_at > ctx.env.block.time {
                poll_engine::execute::cast_vote(
                    cast_vote_ctx,
                    CastVoteParams {
                        poll_id: vote.poll_id.into(),
                        outcome: Default(DefaultVoteOption::from(vote.outcome)),
                        amount: user_available_votes,
                    },
                )?;
            }
        }
    }

    Ok(())
}

pub fn claim(ctx: &mut Context) -> DaoResult<Response> {
    let claims = CLAIMS
        .may_load(ctx.deps.storage, &ctx.info.sender)?
        .unwrap_or_default();

    if claims.is_empty() {
        return Err(DaoError::NothingToClaim);
    }

    let block = ctx.env.block.clone();

    // TODO: this is real brute force, when indexed map is in we should filter smaller data set
    let mut releasable_claims: Vec<Claim> = vec![];
    let remaining_claims = claims
        .into_iter()
        .filter_map(|claim| {
            // TODO: test
            let is_releasable = is_releasable(&claim, &block);
            if is_releasable {
                releasable_claims.push(claim);
                None
            } else {
                // TODO: test
                Some(claim)
            }
        })
        .collect_vec();

    CLAIMS.save(ctx.deps.storage, &ctx.info.sender, &remaining_claims)?;

    let dao_type = DAO_TYPE.load(ctx.deps.storage)?;
    let dao_membership_contract = DAO_MEMBERSHIP_CONTRACT.load(ctx.deps.storage)?;

    let mut submsgs: Vec<SubMsg> = vec![];
    for releasable_claim in releasable_claims {
        match releasable_claim.asset {
            Cw20(msg) => {
                if dao_type != Token {
                    // TODO: test
                    return Err(DaoError::InvalidStakingAsset);
                }
                submsgs.push(SubMsg::new(
                    Asset::cw20(dao_membership_contract.clone(), msg.amount)
                        .transfer_msg(ctx.info.sender.clone())?,
                ))
            }
            Cw721(msg) => {
                if dao_type != Nft {
                    return Err(DaoError::InvalidStakingAsset);
                }
                for token in msg.tokens {
                    submsgs.push(transfer_nft_submsg(
                        dao_membership_contract.to_string(),
                        token,
                        ctx.info.sender.to_string(),
                    )?)
                }
            }
        }
    }

    Ok(Response::new()
        .add_attribute("action", "claim")
        .add_submessages(submsgs))
}

fn transfer_nft_submsg(
    nft_contract: String,
    token_id: String,
    recipient: String,
) -> StdResult<SubMsg> {
    Ok(SubMsg::new(wasm_execute(
        nft_contract,
        &cw721::Cw721ExecuteMsg::TransferNft {
            recipient,
            token_id,
        },
        vec![],
    )?))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> DaoResult<Response> {
    match msg.id {
        DAO_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let addr = deps.api.addr_validate(&contract_address)?;

            DAO_MEMBERSHIP_CONTRACT.save(deps.storage, &addr)?;

            Ok(Response::new())
        }
        _ => Err(DaoError::Std(StdError::generic_err(
            "No such reply ID found",
        ))),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> DaoResult<Binary> {
    let qctx = QueryContext::from(deps, env);

    let response = match msg {
        QueryMsg::DaoInfo {} => to_binary(&query_dao_info(qctx)?)?,
        QueryMsg::MemberInfo(msg) => to_binary(&query_member_info(qctx, msg)?)?,
        QueryMsg::ListMultisigMembers(msg) => to_binary(&query_list_multisig_members(qctx, msg)?)?,
        QueryMsg::AssetWhitelist {} => to_binary(&query_asset_whitelist(qctx)?)?,
        QueryMsg::NftWhitelist {} => to_binary(&query_nft_whitelist(qctx)?)?,
        QueryMsg::Proposal(params) => to_binary(&query_proposal(qctx, params)?)?,
        QueryMsg::Proposals(params) => to_binary(&query_proposals(qctx, params)?)?,
        QueryMsg::ProposalStatus(params) => to_binary(&query_proposal_status(qctx, params)?)?,
        QueryMsg::MemberVote(params) => to_binary(&query_member_vote(qctx, params)?)?,
        QueryMsg::ProposalVotes(params) => to_binary(&query_proposal_votes(qctx, params)?)?,
        QueryMsg::UserStake(params) => to_binary(&query_user_stake(qctx, params)?)?,
        QueryMsg::TotalStakedAmount {} => to_binary(&query_total_staked_amount(qctx)?)?,
        QueryMsg::Claims(params) => to_binary(&query_claims(qctx, params)?)?,
        QueryMsg::ReleasableClaims(params) => to_binary(&query_releasable_claims(qctx, params)?)?,
        QueryMsg::Cw20Treasury {} => to_binary(&query_cw20_treasury(qctx)?)?,
        QueryMsg::NftTreasury {} => to_binary(&query_nft_treasury(qctx)?)?,
    };
    Ok(response)
}

pub fn query_dao_info(qctx: QueryContext) -> DaoResult<DaoInfoResponse> {
    let creation_date = DAO_CREATION_DATE.load(qctx.deps.storage)?;
    let metadata = DAO_METADATA.load(qctx.deps.storage)?;
    let gov_config = DAO_GOV_CONFIG.load(qctx.deps.storage)?;
    let dao_type = DAO_TYPE.load(qctx.deps.storage)?;
    let dao_membership_contract = DAO_MEMBERSHIP_CONTRACT.load(qctx.deps.storage)?;
    let enterprise_factory_contract = ENTERPRISE_FACTORY_CONTRACT.load(qctx.deps.storage)?;
    let dao_code_version = DAO_CODE_VERSION.load(qctx.deps.storage)?;

    Ok(DaoInfoResponse {
        creation_date,
        metadata,
        gov_config,
        dao_type,
        dao_membership_contract,
        enterprise_factory_contract,
        dao_code_version,
    })
}

pub fn query_asset_whitelist(qctx: QueryContext) -> DaoResult<AssetWhitelistResponse> {
    let assets = ASSET_WHITELIST.load(qctx.deps.storage)?;

    Ok(AssetWhitelistResponse { assets })
}

pub fn query_nft_whitelist(qctx: QueryContext) -> DaoResult<NftWhitelistResponse> {
    let nfts = NFT_WHITELIST
        .range(qctx.deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<(Addr, ())>>>()?
        .into_iter()
        .map(|(addr, _)| addr)
        .collect_vec();

    Ok(NftWhitelistResponse { nfts })
}

pub fn query_proposal(qctx: QueryContext, msg: ProposalParams) -> DaoResult<ProposalResponse> {
    let deps = qctx.deps;
    let poll = query_poll(
        &qctx,
        PollParams {
            poll_id: msg.proposal_id,
        },
    )?;

    let proposal = poll_to_proposal_response(deps, &qctx.env, &poll.poll)?;

    Ok(proposal)
}

pub fn query_proposals(qctx: QueryContext, msg: ProposalsParams) -> DaoResult<ProposalsResponse> {
    let deps = qctx.deps;
    let polls = query_polls(
        &qctx,
        PollsParams {
            filter: msg.filter.map(|filter| match filter {
                ProposalStatusFilter::InProgress => PollStatusFilter::InProgress,
                ProposalStatusFilter::Passed => PollStatusFilter::Passed,
                ProposalStatusFilter::Rejected => PollStatusFilter::Rejected,
            }),
            pagination: Pagination {
                start_after: msg.start_after.map(Uint64::from),
                end_at: None,
                limit: msg.limit.map(|limit| limit as u64),
                order_by: None,
            },
        },
    )?;

    let proposals = polls
        .polls
        .into_iter()
        .map(|poll| poll_to_proposal_response(deps, &qctx.env, &poll))
        .collect::<DaoResult<Vec<ProposalResponse>>>()?;

    Ok(ProposalsResponse { proposals })
}

pub fn query_proposal_status(
    qctx: QueryContext,
    msg: ProposalStatusParams,
) -> DaoResult<ProposalStatusResponse> {
    let poll_status = query_poll_status(&qctx, msg.proposal_id)?;

    let status = match poll_status.status {
        PollStatus::InProgress { .. } => ProposalStatus::InProgress,
        PollStatus::Passed { .. } => {
            // TODO: add tests for this
            if is_proposal_executed(qctx.deps.storage, msg.proposal_id, General)? {
                // TODO: add branch for Council, too
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

fn poll_to_proposal_response(deps: Deps, env: &Env, poll: &Poll) -> DaoResult<ProposalResponse> {
    let actions_opt = get_proposal_actions(deps.storage, poll.id, General)?; // TODO: add branch for Council type as well

    let actions = match actions_opt {
        None => {
            // TODO: test
            return Err(PollError::PollNotFound {
                poll_id: poll.id.into(),
            }
            .into());
        }
        Some(actions) => actions,
    };

    let status = match poll.status {
        PollStatus::InProgress { .. } => ProposalStatus::InProgress,
        PollStatus::Passed { .. } => ProposalStatus::Passed,
        PollStatus::Rejected { .. } => ProposalStatus::Rejected,
    };

    let proposal = Proposal {
        id: poll.id,
        title: poll.label.clone(),
        description: poll.description.clone(),
        status,
        started_at: poll.started_at,
        expires: Expiration::AtTime(poll.ends_at),
        proposal_actions: actions,
    };

    let info = PROPOSAL_INFOS.load(deps.storage, poll.id)?;

    let dao_type = DAO_TYPE.load(deps.storage)?;

    let total_votes_available = match info.executed_at {
        Some(block) => match proposal.expires {
            Expiration::AtHeight(height) => {
                // TODO: test
                total_available_votes_at_height(dao_type, deps.storage, min(height, block.height))?
            }
            Expiration::AtTime(time) => {
                total_available_votes_at_time(dao_type, deps.storage, min(time, block.time))?
            }
            Expiration::Never { .. } => {
                // TODO: test
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
    };

    Ok(ProposalResponse {
        proposal,
        results: poll.results.clone(),
        total_votes_available,
    })
}

fn total_available_votes_at_height(
    dao_type: DaoType,
    store: &dyn Storage,
    height: u64,
) -> StdResult<Uint128> {
    // TODO: tests
    match dao_type {
        Token | Nft => load_total_staked_at_height(store, height),
        Multisig => load_total_multisig_weight_at_height(store, height),
    }
}

fn total_available_votes_at_time(
    dao_type: DaoType,
    store: &dyn Storage,
    time: Timestamp,
) -> StdResult<Uint128> {
    match dao_type {
        Token | Nft => load_total_staked_at_time(store, time),
        Multisig => load_total_multisig_weight_at_time(store, time),
    }
}

fn current_total_available_votes(dao_type: DaoType, store: &dyn Storage) -> StdResult<Uint128> {
    match dao_type {
        Token | Nft => load_total_staked(store),
        Multisig => load_total_multisig_weight(store),
    }
}

// TODO: tests
pub fn query_member_vote(
    qctx: QueryContext,
    params: MemberVoteParams,
) -> DaoResult<MemberVoteResponse> {
    let vote = query_poll_voter(
        &qctx,
        PollVoterParams {
            poll_id: params.proposal_id.into(),
            voter_addr: params.member,
        },
    )?;

    Ok(MemberVoteResponse { vote: vote.vote })
}

// TODO: tests
pub fn query_proposal_votes(
    qctx: QueryContext,
    params: ProposalVotesParams,
) -> DaoResult<ProposalVotesResponse> {
    let poll_voters = query_poll_voters(
        &qctx,
        PollVotersParams {
            poll_id: params.proposal_id,
            pagination: Pagination {
                start_after: params.start_after,
                end_at: None,
                limit: params.limit.map(|limit| limit as u64),
                order_by: None,
            },
        },
    )?;

    Ok(ProposalVotesResponse {
        votes: poll_voters.votes,
    })
}

pub fn query_member_info(
    qctx: QueryContext,
    msg: QueryMemberInfoMsg,
) -> DaoResult<MemberInfoResponse> {
    let dao_type = DAO_TYPE.load(qctx.deps.storage)?;

    let voting_power = match dao_type {
        Token => calculate_token_member_voting_power(qctx, msg.member_address)?, // TODO: test
        Nft => calculate_nft_member_voting_power(qctx, msg.member_address)?,
        Multisig => calculate_multisig_member_voting_power(qctx, msg.member_address)?,
    };

    Ok(MemberInfoResponse { voting_power })
}

fn calculate_token_member_voting_power(qctx: QueryContext, member: String) -> DaoResult<Decimal> {
    let total_staked = load_total_staked(qctx.deps.storage)?;

    if total_staked == Uint128::zero() {
        Ok(Decimal::zero())
    } else {
        let member = qctx.deps.api.addr_validate(&member)?;
        let user_staked = CW20_STAKES
            .may_load(qctx.deps.storage, member)?
            .unwrap_or_default();

        Ok(Decimal::from_ratio(user_staked, total_staked))
    }
}

fn calculate_nft_member_voting_power(qctx: QueryContext, member: String) -> DaoResult<Decimal> {
    let total_staked = load_total_staked(qctx.deps.storage)?;

    if total_staked == Uint128::zero() {
        Ok(Decimal::zero())
    } else {
        let member = qctx.deps.api.addr_validate(&member)?;

        let user_staked = load_all_nft_stakes_for_user(qctx.deps.storage, member)?;

        Ok(Decimal::from_ratio(user_staked.len() as u128, total_staked))
    }
}

fn calculate_multisig_member_voting_power(
    qctx: QueryContext,
    member: String,
) -> DaoResult<Decimal> {
    let total_weight = load_total_multisig_weight(qctx.deps.storage)?;

    if total_weight == Uint128::zero() {
        Ok(Decimal::zero())
    } else {
        let member_addr = qctx.deps.api.addr_validate(&member)?;
        let voter_weight = MULTISIG_MEMBERS
            .may_load(qctx.deps.storage, member_addr)?
            .unwrap_or_default();

        Ok(Decimal::from_ratio(voter_weight, total_weight))
    }
}

// TODO: tests
pub fn query_list_multisig_members(
    qctx: QueryContext,
    msg: ListMultisigMembersMsg,
) -> DaoResult<MultisigMembersResponse> {
    let dao_type = DAO_TYPE.load(qctx.deps.storage)?;

    if dao_type != Multisig {
        return Err(UnsupportedOperationForDaoType {
            dao_type: dao_type.to_string(),
        });
    }

    let start_after = msg
        .start_after
        .map(|addr| qctx.deps.api.addr_validate(&addr))
        .transpose()?
        .map(Bound::exclusive);

    let members = MULTISIG_MEMBERS
        .range(qctx.deps.storage, start_after, None, Ascending)
        .take(
            msg.limit
                .unwrap_or(DEFAULT_QUERY_LIMIT)
                .min(MAX_QUERY_LIMIT) as usize,
        )
        .collect::<StdResult<Vec<(Addr, Uint128)>>>()?
        .into_iter()
        .map(|(addr, weight)| MultisigMember {
            address: addr.to_string(),
            weight,
        })
        .collect_vec();

    Ok(MultisigMembersResponse { members })
}

fn get_user_available_votes(qctx: QueryContext, user: Addr) -> DaoResult<Uint128> {
    let dao_type = DAO_TYPE.load(qctx.deps.storage)?;

    let user_available_votes = match dao_type {
        Token => get_user_staked_tokens(qctx, user)?.amount,
        Nft => get_user_staked_nfts(qctx, user)?.amount,
        Multisig => MULTISIG_MEMBERS
            .may_load(qctx.deps.storage, user)?
            .unwrap_or_default(),
    };

    Ok(user_available_votes)
}

pub fn query_user_stake(
    qctx: QueryContext,
    params: UserStakeParams,
) -> DaoResult<UserStakeResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    let dao_type = DAO_TYPE.load(qctx.deps.storage)?;

    let user_stake: UserStake = match dao_type {
        Token => UserStake::Token(get_user_staked_tokens(qctx, user)?),
        Nft => UserStake::Nft(get_user_staked_nfts(qctx, user)?),
        Multisig => UserStake::None, // TODO: test
    };

    Ok(UserStakeResponse { user_stake })
}

fn get_user_staked_tokens(qctx: QueryContext, user: Addr) -> DaoResult<TokenUserStake> {
    let staked_amount = CW20_STAKES
        .may_load(qctx.deps.storage, user)?
        .unwrap_or_default();
    Ok(TokenUserStake {
        amount: staked_amount,
    })
}

fn get_user_staked_nfts(qctx: QueryContext, user: Addr) -> DaoResult<NftUserStake> {
    let staked_tokens = load_all_nft_stakes_for_user(qctx.deps.storage, user)?
        .into_iter()
        .map(|stake| stake.token_id)
        .collect_vec();
    let amount = staked_tokens.len() as u128;

    Ok(NftUserStake {
        tokens: staked_tokens,
        amount: amount.into(),
    })
}

pub fn query_total_staked_amount(qctx: QueryContext) -> DaoResult<TotalStakedAmountResponse> {
    let total_staked_amount = load_total_staked(qctx.deps.storage)?;

    Ok(TotalStakedAmountResponse {
        total_staked_amount,
    })
}

pub fn query_claims(qctx: QueryContext, params: ClaimsParams) -> DaoResult<ClaimsResponse> {
    let sender = qctx.deps.api.addr_validate(&params.owner)?;

    let claims = CLAIMS
        .may_load(qctx.deps.storage, &sender)?
        .unwrap_or_default();

    Ok(ClaimsResponse { claims })
}

pub fn query_releasable_claims(
    qctx: QueryContext,
    params: ClaimsParams,
) -> DaoResult<ClaimsResponse> {
    let block = qctx.env.block.clone();
    let claims = query_claims(qctx, params)?;

    // TODO: this is real brute force, when indexed map is in we should filter smaller data set
    let releasable_claims = claims
        .claims
        .into_iter()
        .filter(|claim| is_releasable(claim, &block))
        .collect_vec();

    Ok(ClaimsResponse {
        claims: releasable_claims,
    })
}

// TODO: tests
fn query_cw20_treasury(qctx: QueryContext) -> DaoResult<AssetTreasuryResponse> {
    let enterprise_factory = ENTERPRISE_FACTORY_CONTRACT.load(qctx.deps.storage)?;
    let global_asset_whitelist: AssetWhitelistResponse = qctx
        .deps
        .querier
        .query_wasm_smart(enterprise_factory, &GlobalAssetWhitelist {})?;

    // TODO: add clone() to qctx?
    let qctx2 = QueryContext::from(qctx.deps, qctx.env.clone());
    let mut asset_whitelist = query_asset_whitelist(qctx2)?.assets;

    for asset in global_asset_whitelist.assets {
        // TODO: suboptimal, use maps or sth
        if !asset_whitelist.contains(&asset) {
            asset_whitelist.push(asset);
        }
    }

    // add 'uluna' to the asset whitelist, if not already present
    let uluna = AssetInfo::native("uluna");
    if !asset_whitelist.contains(&uluna) {
        asset_whitelist.push(uluna);
    }

    let dao_membership_contract = DAO_MEMBERSHIP_CONTRACT.load(qctx.deps.storage)?;

    // if DAO is of Token type, add its own token to the asset whitelist, if not already present
    let dao_type = DAO_TYPE.load(qctx.deps.storage)?;
    if dao_type == Token {
        let dao_token = AssetInfo::cw20(dao_membership_contract.clone());
        if !asset_whitelist.contains(&dao_token) {
            asset_whitelist.push(dao_token);
        }
    }

    let dao_address = qctx.env.contract.address.as_ref();

    let assets = asset_whitelist
        .into_iter()
        .map(|asset| {
            // subtract staked and deposited tokens, if the asset is DAO's membership token
            let subtract_amount = if let AssetInfo::Cw20(token) = &asset {
                if token == &dao_membership_contract {
                    let staked = load_total_staked(qctx.deps.storage)?;
                    let deposited = TOTAL_DEPOSITS.load(qctx.deps.storage)?;

                    staked.add(deposited)
                } else {
                    Uint128::zero()
                }
            } else {
                Uint128::zero()
            };
            asset
                .query_balance(&qctx.deps.querier, dao_address)
                .map(|balance| Asset::new(asset, balance.sub(subtract_amount)))
        })
        .collect::<StdResult<Vec<Asset>>>()?;

    Ok(AssetTreasuryResponse { assets })
}

// TODO: tests
fn query_nft_treasury(qctx: QueryContext) -> DaoResult<NftTreasuryResponse> {
    let enterprise_factory = ENTERPRISE_FACTORY_CONTRACT.load(qctx.deps.storage)?;
    let global_nft_whitelist: NftWhitelistResponse = qctx
        .deps
        .querier
        .query_wasm_smart(enterprise_factory, &GlobalNftWhitelist {})?;

    // TODO: add clone() to qctx?
    let qctx2 = QueryContext::from(qctx.deps, qctx.env.clone());
    let mut nft_whitelist = query_nft_whitelist(qctx2)?.nfts;

    for nft in global_nft_whitelist.nfts {
        // TODO: suboptimal, use maps or sth
        if !nft_whitelist.contains(&nft) {
            nft_whitelist.push(nft);
        }
    }

    let dao_membership_contract = DAO_MEMBERSHIP_CONTRACT.load(qctx.deps.storage)?;

    // if the DAO has an NFT membership, remove it from the whitelist, as it is handled separately
    let nft_whitelist = nft_whitelist
        .into_iter()
        .filter(|nft| nft != &dao_membership_contract)
        .collect_vec();

    let dao_address = qctx.env.contract.address.as_ref();

    let mut nfts = nft_whitelist
        .into_iter()
        .map(|nft| {
            get_all_owner_tokens(&qctx, nft.as_ref(), dao_address).map(|all_tokens| NftCollection {
                nft_address: nft,
                token_ids: all_tokens,
            })
        })
        .collect::<DaoResult<Vec<NftCollection>>>()?;

    // if DAO is of Nft type, add its own NFT to the NFT whitelist, if not already present
    let dao_type = DAO_TYPE.load(qctx.deps.storage)?;
    if dao_type == Nft {
        let all_tokens =
            get_all_owner_tokens(&qctx, dao_membership_contract.as_str(), dao_address)?;

        // filter out tokens staked by users
        let dao_owned_tokens = all_tokens
            .into_iter()
            .filter(|token| !NFT_STAKES().has(qctx.deps.storage, token.to_string()))
            .collect_vec();

        let dao_nft_collection = NftCollection {
            nft_address: dao_membership_contract,
            token_ids: dao_owned_tokens,
        };

        // add DAO NFTs to the front
        nfts.insert(0, dao_nft_collection);
    }

    Ok(NftTreasuryResponse { nfts })
}

fn get_all_owner_tokens(
    qctx: &QueryContext,
    nft_addr: &str,
    owner: &str,
) -> DaoResult<Vec<String>> {
    let mut tokens_response = get_owner_tokens(qctx, nft_addr, owner, None)?;

    // TODO: try to avoid this double assignment by using do-while
    let mut tokens = tokens_response.tokens;
    let mut owner_tokens = tokens.clone();
    let mut last_token = tokens.last();
    while !tokens.is_empty() && last_token.is_some() {
        tokens_response = get_owner_tokens(qctx, nft_addr, owner, last_token)?;
        tokens = tokens_response.tokens;
        tokens
            .iter()
            .for_each(|token| owner_tokens.push(token.to_string()));
        last_token = tokens.last();
    }

    Ok(owner_tokens)
}

fn get_owner_tokens(
    qctx: &QueryContext,
    nft_addr: &str,
    owner: &str,
    start_after: Option<&String>,
) -> DaoResult<TokensResponse> {
    let query_owner_tokens: cw721_base::QueryMsg<Empty> = cw721_base::QueryMsg::Tokens {
        owner: owner.to_string(),
        start_after: start_after.cloned(),
        limit: Some(u32::MAX),
    };
    Ok(qctx
        .deps
        .querier
        .query_wasm_smart(nft_addr.to_string(), &query_owner_tokens)?)
}

fn is_releasable(claim: &Claim, block_info: &BlockInfo) -> bool {
    match claim.release_at {
        ReleaseAt::Timestamp(timestamp) => block_info.time >= timestamp,
        ReleaseAt::Height(height) => block_info.height >= height.u64(),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> DaoResult<Response> {
    Ok(Response::new())
}
