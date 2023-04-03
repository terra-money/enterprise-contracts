use crate::cw20::{Cw20InstantiateMsg, InstantiateMarketingInfo};
use crate::cw3::{Cw3ListVoters, Cw3VoterListResponse};
use crate::cw721::{Cw721InstantiateMsg, Cw721QueryMsg};
use crate::multisig::{
    load_total_multisig_weight, load_total_multisig_weight_at_height,
    load_total_multisig_weight_at_time, save_total_multisig_weight, MULTISIG_MEMBERS,
};
use crate::nft_staking;
use crate::nft_staking::{load_all_nft_stakes_for_user, save_nft_stake, NftStake};
use crate::proposals::{
    get_proposal_actions, is_proposal_executed, set_proposal_executed, ProposalInfo,
    PROPOSAL_INFOS, TOTAL_DEPOSITS,
};
use crate::staking::{
    load_total_staked, load_total_staked_at_height, load_total_staked_at_time, save_total_staked,
    CW20_STAKES,
};
use crate::state::{
    add_claim, State, ASSET_WHITELIST, CLAIMS, DAO_CODE_VERSION, DAO_COUNCIL, DAO_CREATION_DATE,
    DAO_GOV_CONFIG, DAO_MEMBERSHIP_CONTRACT, DAO_METADATA, DAO_TYPE, ENTERPRISE_FACTORY_CONTRACT,
    ENTERPRISE_GOVERNANCE_CONTRACT, FUNDS_DISTRIBUTOR_CONTRACT, NFT_WHITELIST, STATE,
};
use crate::validate::{
    apply_gov_config_changes, normalize_asset_whitelist, validate_dao_council,
    validate_dao_gov_config, validate_deposit, validate_existing_dao_contract,
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
use cw721::TokensResponse;
use cw_asset::{Asset, AssetInfo, AssetInfoBase};
use cw_storage_plus::Bound;
use cw_utils::{parse_reply_instantiate_data, Duration, Expiration};
use enterprise_factory_api::msg::QueryMsg::{GlobalAssetWhitelist, GlobalNftWhitelist};
use enterprise_protocol::api::ClaimAsset::{Cw20, Cw721};
use enterprise_protocol::api::DaoType::{Multisig, Nft};
use enterprise_protocol::api::ModifyValue::Change;
use enterprise_protocol::api::ProposalType::{Council, General};
use enterprise_protocol::api::{
    AssetTreasuryResponse, AssetWhitelistResponse, CastVoteMsg, Claim, ClaimsParams,
    ClaimsResponse, CreateProposalMsg, Cw20ClaimAsset, Cw721ClaimAsset, DaoGovConfig,
    DaoInfoResponse, DaoMembershipInfo, DaoType, DistributeFundsMsg, ExecuteMsgsMsg,
    ExecuteProposalMsg, ExistingDaoMembershipMsg, ListMultisigMembersMsg, MemberInfoResponse,
    MemberVoteParams, MemberVoteResponse, ModifyMultisigMembershipMsg, MultisigMember,
    MultisigMembersResponse, NewDaoMembershipMsg, NewMembershipInfo, NewMultisigMembershipInfo,
    NewNftMembershipInfo, NewTokenMembershipInfo, NftCollection, NftTokenId, NftTreasuryResponse,
    NftUserStake, NftWhitelistResponse, Proposal, ProposalAction, ProposalActionType,
    ProposalDeposit, ProposalId, ProposalParams, ProposalResponse, ProposalStatus,
    ProposalStatusFilter, ProposalStatusParams, ProposalStatusResponse, ProposalType,
    ProposalVotesParams, ProposalVotesResponse, ProposalsParams, ProposalsResponse,
    QueryMemberInfoMsg, ReceiveNftMsg, ReleaseAt, RequestFundingFromDaoMsg,
    TalisFriendlyTokensResponse, TokenUserStake, TotalStakedAmountResponse, UnstakeMsg,
    UpdateAssetWhitelistMsg, UpdateCouncilMsg, UpdateGovConfigMsg, UpdateMetadataMsg,
    UpdateMinimumWeightForRewardsMsg, UpdateNftWhitelistMsg, UpgradeDaoMsg, UserStake,
    UserStakeParams, UserStakeResponse,
};
use enterprise_protocol::error::DaoError::{
    DuplicateMultisigMember, InsufficientStakedAssets, InvalidCosmosMessage, NoVotesAvailable,
    NotMultisigMember, NothingToClaim, ProposalAlreadyExecuted, Std,
    UnsupportedCouncilProposalAction, WrongProposalType, ZeroInitialDaoBalance,
};
use enterprise_protocol::error::{DaoError, DaoResult};
use enterprise_protocol::msg::{
    Cw20HookMsg, Cw721HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use funds_distributor_api::api::{
    UpdateMinimumEligibleWeightMsg, UpdateUserWeightsMsg, UserWeight,
};
use funds_distributor_api::msg::Cw20HookMsg::Distribute;
use funds_distributor_api::msg::ExecuteMsg::DistributeNative;
use nft_staking::NFT_STAKES;
use poll_engine_api::api::{
    CastVoteParams, CreatePollParams, EndPollParams, Poll, PollId, PollParams, PollRejectionReason,
    PollResponse, PollStatus, PollStatusFilter, PollStatusResponse, PollVoterParams,
    PollVoterResponse, PollVotersParams, PollVotersResponse, PollsParams, PollsResponse,
    UpdateVotesParams, VotingScheme,
};
use poll_engine_api::error::PollError::PollInProgress;
use std::cmp::min;
use std::ops::{Add, Not, Sub};
use DaoError::{
    CustomError, InvalidStakingAsset, NoDaoCouncil, NoNftTokenStaked, NoSuchProposal, Unauthorized,
    UnsupportedOperationForDaoType, ZeroInitialWeightMember,
};
use DaoMembershipInfo::{Existing, New};
use DaoType::Token;
use Duration::{Height, Time};
use NewMembershipInfo::{NewMultisig, NewNft, NewToken};
use PollRejectionReason::{IsVetoOutcome, QuorumNotReached};
use ProposalAction::{
    DistributeFunds, ExecuteMsgs, ModifyMultisigMembership, RequestFundingFromDao,
    UpdateAssetWhitelist, UpdateCouncil, UpdateGovConfig, UpdateMetadata,
    UpdateMinimumWeightForRewards, UpdateNftWhitelist, UpgradeDao,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DAO_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 1;
pub const ENTERPRISE_GOVERNANCE_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 2;
pub const FUNDS_DISTRIBUTOR_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 3;
pub const CREATE_POLL_REPLY_ID: u64 = 4;
pub const END_POLL_REPLY_ID: u64 = 5;

pub const CODE_VERSION: u8 = 4;

pub const DEFAULT_QUERY_LIMIT: u8 = 50;
pub const MAX_QUERY_LIMIT: u8 = 100;

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

    let dao_council = validate_dao_council(deps.as_ref(), msg.dao_council.clone())?;

    STATE.save(
        deps.storage,
        &State {
            proposal_being_created: None,
            proposal_being_executed: None,
        },
    )?;

    DAO_CREATION_DATE.save(deps.storage, &env.block.time)?;

    DAO_METADATA.save(deps.storage, &msg.dao_metadata)?;
    DAO_GOV_CONFIG.save(deps.storage, &msg.dao_gov_config)?;
    DAO_COUNCIL.save(deps.storage, &dao_council)?;
    ENTERPRISE_FACTORY_CONTRACT.save(
        deps.storage,
        &deps.api.addr_validate(&msg.enterprise_factory_contract)?,
    )?;
    DAO_CODE_VERSION.save(deps.storage, &CODE_VERSION.into())?;

    let normalized_asset_whitelist =
        normalize_asset_whitelist(deps.as_ref(), &msg.asset_whitelist.unwrap_or_default())?;
    ASSET_WHITELIST.save(deps.storage, &normalized_asset_whitelist)?;

    for nft in &msg.nft_whitelist.unwrap_or_default() {
        NFT_WHITELIST.save(deps.storage, deps.api.addr_validate(nft.as_ref())?, &())?;
    }

    save_total_staked(deps.storage, &Uint128::zero(), &env.block)?;
    TOTAL_DEPOSITS.save(deps.storage, &Uint128::zero())?;

    // instantiate the governance contract
    let instantiate_governance_contract_submsg = SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.enterprise_governance_code_id,
            msg: to_binary(&enterprise_governance_api::msg::InstantiateMsg {
                enterprise_contract: env.contract.address.to_string(),
            })?,
            funds: vec![],
            label: "Governance contract".to_string(),
        }),
        ENTERPRISE_GOVERNANCE_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    let ctx = Context { deps, env, info };

    let mut submessages = match msg.dao_membership_info {
        New(membership) => instantiate_new_membership_dao(
            ctx,
            membership,
            msg.funds_distributor_code_id,
            msg.minimum_weight_for_rewards,
        )?,
        Existing(membership) => instantiate_existing_membership_dao(
            ctx,
            membership,
            msg.funds_distributor_code_id,
            msg.minimum_weight_for_rewards,
        )?,
    };

    submessages.push(instantiate_governance_contract_submsg);

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_submessages(submessages))
}

fn instantiate_funds_distributor_submsg(
    ctx: &Context,
    funds_distributor_code_id: u64,
    minimum_weight_for_rewards: Option<Uint128>,
    initial_weights: Vec<UserWeight>,
) -> DaoResult<SubMsg> {
    let instantiate_funds_distributor_contract_submsg = SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: Some(ctx.env.contract.address.to_string()),
            code_id: funds_distributor_code_id,
            msg: to_binary(&funds_distributor_api::msg::InstantiateMsg {
                enterprise_contract: ctx.env.contract.address.to_string(),
                initial_weights,
                minimum_eligible_weight: minimum_weight_for_rewards,
            })?,
            funds: vec![],
            label: "Funds distributor contract".to_string(),
        }),
        FUNDS_DISTRIBUTOR_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    Ok(instantiate_funds_distributor_contract_submsg)
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
    funds_distributor_code_id: u64,
    minimum_weight_for_rewards: Option<Uint128>,
) -> DaoResult<Vec<SubMsg>> {
    match membership.membership_info {
        NewToken(info) => instantiate_new_token_dao(
            ctx,
            *info,
            membership.membership_contract_code_id,
            funds_distributor_code_id,
            minimum_weight_for_rewards,
        ),
        NewNft(info) => instantiate_new_nft_dao(
            ctx,
            info,
            membership.membership_contract_code_id,
            funds_distributor_code_id,
            minimum_weight_for_rewards,
        ),
        NewMultisig(info) => instantiate_new_multisig_dao(
            ctx,
            info,
            funds_distributor_code_id,
            minimum_weight_for_rewards,
        ),
    }
}

fn instantiate_new_token_dao(
    ctx: Context,
    info: NewTokenMembershipInfo,
    cw20_code_id: u64,
    funds_distributor_code_id: u64,
    minimum_weight_for_rewards: Option<Uint128>,
) -> DaoResult<Vec<SubMsg>> {
    if let Some(initial_dao_balance) = info.initial_dao_balance {
        if initial_dao_balance == Uint128::zero() {
            return Err(ZeroInitialDaoBalance);
        }
    }

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

    let initial_balances = match info.initial_dao_balance {
        None => info.initial_token_balances,
        Some(initial_dao_balance) => {
            let mut token_balances = info.initial_token_balances;
            token_balances.push(Cw20Coin {
                address: ctx.env.contract.address.to_string(),
                amount: initial_dao_balance,
            });
            token_balances
        }
    };

    let create_token_msg = Cw20InstantiateMsg {
        name: info.token_name.clone(),
        symbol: info.token_symbol,
        decimals: info.token_decimals,
        initial_balances,
        mint: info.token_mint.or_else(|| {
            Some(MinterResponse {
                minter: ctx.env.contract.address.to_string(),
                cap: None,
            })
        }),
        marketing,
    };

    let instantiate_dao_token_submsg = SubMsg::reply_on_success(
        wasm_instantiate(cw20_code_id, &create_token_msg, vec![], info.token_name)?,
        DAO_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    Ok(vec![
        instantiate_dao_token_submsg,
        instantiate_funds_distributor_submsg(
            &ctx,
            funds_distributor_code_id,
            minimum_weight_for_rewards,
            vec![],
        )?,
    ])
}

fn instantiate_new_multisig_dao(
    ctx: Context,
    info: NewMultisigMembershipInfo,
    funds_distributor_code_id: u64,
    minimum_weight_for_rewards: Option<Uint128>,
) -> DaoResult<Vec<SubMsg>> {
    DAO_TYPE.save(ctx.deps.storage, &Multisig)?;

    let mut total_weight = Uint128::zero();

    let mut initial_weights: Vec<UserWeight> = vec![];

    for member in info.multisig_members.into_iter() {
        if member.weight == Uint128::zero() {
            return Err(ZeroInitialWeightMember);
        }

        let member_addr = ctx.deps.api.addr_validate(&member.address)?;

        if MULTISIG_MEMBERS.has(ctx.deps.storage, member_addr.clone()) {
            return Err(DuplicateMultisigMember);
        }

        MULTISIG_MEMBERS.save(ctx.deps.storage, member_addr, &member.weight)?;

        initial_weights.push(UserWeight {
            user: member.address,
            weight: member.weight,
        });

        total_weight = total_weight.add(member.weight);
    }

    save_total_multisig_weight(ctx.deps.storage, total_weight, &ctx.env.block)?;

    DAO_MEMBERSHIP_CONTRACT.save(ctx.deps.storage, &ctx.env.contract.address)?;

    Ok(vec![instantiate_funds_distributor_submsg(
        &ctx,
        funds_distributor_code_id,
        minimum_weight_for_rewards,
        initial_weights,
    )?])
}

fn instantiate_new_nft_dao(
    ctx: Context,
    info: NewNftMembershipInfo,
    cw721_code_id: u64,
    funds_distributor_code_id: u64,
    minimum_weight_for_rewards: Option<Uint128>,
) -> DaoResult<Vec<SubMsg>> {
    DAO_TYPE.save(ctx.deps.storage, &Nft)?;

    let minter = match info.minter {
        None => ctx.env.contract.address.to_string(),
        Some(minter) => minter,
    };
    let instantiate_msg = Cw721InstantiateMsg {
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
    Ok(vec![
        submsg,
        instantiate_funds_distributor_submsg(
            &ctx,
            funds_distributor_code_id,
            minimum_weight_for_rewards,
            vec![],
        )?,
    ])
}

fn instantiate_existing_membership_dao(
    ctx: Context,
    membership: ExistingDaoMembershipMsg,
    funds_distributor_code_id: u64,
    minimum_weight_for_rewards: Option<Uint128>,
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

    let mut initial_weights: Vec<UserWeight> = vec![];

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
                let query_msg = Cw3ListVoters {
                    start_after: last_voter.clone(),
                    limit: None,
                };

                last_voter = None;

                let voters: Cw3VoterListResponse = ctx
                    .deps
                    .querier
                    .query_wasm_smart(&membership.membership_contract_addr, &query_msg)?;

                for voter in voters.voters {
                    last_voter = Some(voter.addr.clone());

                    let voter_addr = ctx.deps.api.addr_validate(&voter.addr)?;
                    MULTISIG_MEMBERS.save(ctx.deps.storage, voter_addr, &voter.weight.into())?;

                    initial_weights.push(UserWeight {
                        user: voter.addr,
                        weight: voter.weight.into(),
                    });

                    total_weight = total_weight.add(Uint128::from(voter.weight));
                }

                last_voter.is_some()
            } {}

            save_total_multisig_weight(ctx.deps.storage, total_weight, &ctx.env.block)?;
        }
    }

    Ok(vec![instantiate_funds_distributor_submsg(
        &ctx,
        funds_distributor_code_id,
        minimum_weight_for_rewards,
        initial_weights,
    )?])
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> DaoResult<Response> {
    let mut ctx = Context { deps, env, info };
    match msg {
        ExecuteMsg::CreateProposal(msg) => create_proposal(&mut ctx, msg, None),
        ExecuteMsg::CreateCouncilProposal(msg) => create_council_proposal(&mut ctx, msg),
        ExecuteMsg::CastVote(msg) => cast_vote(&mut ctx, msg),
        ExecuteMsg::CastCouncilVote(msg) => cast_council_vote(&mut ctx, msg),
        ExecuteMsg::ExecuteProposal(msg) => execute_proposal(&mut ctx, msg),
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

    let dao_type = DAO_TYPE.load(ctx.deps.storage)?;

    let create_poll_submsg = create_poll(ctx, gov_config, msg, deposit, General)?;

    let response = Response::new()
        .add_attribute("action", "create_proposal")
        .add_attribute("dao_address", ctx.env.contract.address.to_string())
        .add_submessage(create_poll_submsg);

    match dao_type {
        Token => Ok(response),
        Nft => {
            if !user_has_nfts_staked(ctx)? && !user_holds_nft(ctx)? {
                return Err(DaoError::NotNftOwner {});
            }

            Ok(response)
        }
        Multisig => {
            let member_weight = MULTISIG_MEMBERS
                .may_load(ctx.deps.storage, ctx.info.sender.clone())?
                .unwrap_or_default();
            if member_weight == Uint128::zero() {
                return Err(NotMultisigMember {});
            }

            Ok(response)
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
    let tokens: TalisFriendlyTokensResponse = ctx
        .deps
        .querier
        .query_wasm_smart(cw721_contract.to_string(), &query_tokens_msg)?;

    Ok(tokens.to_tokens_response()?.tokens.is_empty().not())
}

fn create_council_proposal(ctx: &mut Context, msg: CreateProposalMsg) -> DaoResult<Response> {
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

            let gov_config = DAO_GOV_CONFIG.load(ctx.deps.storage)?;

            let council_gov_config = DaoGovConfig {
                quorum: dao_council.quorum,
                threshold: dao_council.threshold,
                ..gov_config
            };

            let create_poll_submsg = create_poll(ctx, council_gov_config, msg, None, Council)?;

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
    gov_config: DaoGovConfig,
    msg: CreateProposalMsg,
    deposit: Option<ProposalDeposit>,
    proposal_type: ProposalType,
) -> DaoResult<SubMsg> {
    let ends_at = ctx.env.block.time.plus_seconds(gov_config.vote_duration);

    let governance_contract = ENTERPRISE_GOVERNANCE_CONTRACT.load(ctx.deps.storage)?;
    let create_poll_submsg = SubMsg::reply_on_success(
        wasm_execute(
            governance_contract.to_string(),
            &enterprise_governance_api::msg::ExecuteMsg::CreatePoll(CreatePollParams {
                proposer: ctx.info.sender.to_string(),
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

fn cast_vote(ctx: &mut Context, msg: CastVoteMsg) -> DaoResult<Response> {
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

    let governance_contract = ENTERPRISE_GOVERNANCE_CONTRACT.load(ctx.deps.storage)?;

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

fn cast_council_vote(ctx: &mut Context, msg: CastVoteMsg) -> DaoResult<Response> {
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

            let governance_contract = ENTERPRISE_GOVERNANCE_CONTRACT.load(ctx.deps.storage)?;

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

fn execute_proposal(ctx: &mut Context, msg: ExecuteProposalMsg) -> DaoResult<Response> {
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
) -> DaoResult<Vec<SubMsg>> {
    let proposal_info = PROPOSAL_INFOS
        .may_load(deps.storage, proposal_id)?
        .ok_or(NoSuchProposal)?;

    return_deposit_submsgs(deps, proposal_info.proposal_deposit)
}

fn return_deposit_submsgs(
    deps: DepsMut,
    deposit: Option<ProposalDeposit>,
) -> DaoResult<Vec<SubMsg>> {
    match deposit {
        None => Ok(vec![]),
        Some(deposit) => {
            let membership_contract = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

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
) -> DaoResult<Vec<SubMsg>> {
    let qctx = QueryContext::from(ctx.deps.as_ref(), ctx.env.clone());
    let poll = query_poll(&qctx, msg.proposal_id)?;

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

    let allow_early_ending = match proposal_type {
        General => {
            let gov_config = DAO_GOV_CONFIG.load(ctx.deps.storage)?;
            gov_config.allow_early_proposal_execution
        }
        Council => true,
    };

    let governance_contract = ENTERPRISE_GOVERNANCE_CONTRACT.load(ctx.deps.storage)?;
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

fn resolve_ended_proposal(ctx: &mut Context, proposal_id: ProposalId) -> DaoResult<Vec<SubMsg>> {
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
            let mut submsgs = execute_proposal_actions(ctx, proposal_id)?;
            let mut return_deposit_submsgs =
                return_proposal_deposit_submsgs(ctx.deps.branch(), proposal_id)?;
            submsgs.append(&mut return_deposit_submsgs);

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

fn execute_proposal_actions(ctx: &mut Context, proposal_id: ProposalId) -> DaoResult<Vec<SubMsg>> {
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

fn update_metadata(deps: DepsMut, msg: UpdateMetadataMsg) -> DaoResult<Vec<SubMsg>> {
    let mut metadata = DAO_METADATA.load(deps.storage)?;

    if let Change(name) = msg.name {
        metadata.name = name;
    }

    if let Change(description) = msg.description {
        metadata.description = description;
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

    DAO_METADATA.save(deps.storage, &metadata)?;

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
        .collect();

    Ok(submsgs)
}

fn update_gov_config(ctx: &mut Context, msg: UpdateGovConfigMsg) -> DaoResult<Vec<SubMsg>> {
    let gov_config = DAO_GOV_CONFIG.load(ctx.deps.storage)?;

    let updated_gov_config = apply_gov_config_changes(gov_config, &msg);

    let dao_type = DAO_TYPE.load(ctx.deps.storage)?;

    validate_dao_gov_config(&dao_type, &updated_gov_config)?;

    DAO_GOV_CONFIG.save(ctx.deps.storage, &updated_gov_config)?;

    Ok(vec![])
}

fn update_council(ctx: &mut Context, msg: UpdateCouncilMsg) -> DaoResult<Vec<SubMsg>> {
    let dao_council = validate_dao_council(ctx.deps.as_ref(), msg.dao_council)?;

    DAO_COUNCIL.save(ctx.deps.storage, &dao_council)?;

    Ok(vec![])
}

fn update_asset_whitelist(deps: DepsMut, msg: UpdateAssetWhitelistMsg) -> DaoResult<Vec<SubMsg>> {
    let mut asset_whitelist = ASSET_WHITELIST.load(deps.storage)?;

    for add in msg.add {
        asset_whitelist.push(add);
    }

    asset_whitelist = asset_whitelist
        .into_iter()
        .filter(|asset| !msg.remove.contains(asset))
        .collect();

    let normalized_asset_whitelist = normalize_asset_whitelist(deps.as_ref(), &asset_whitelist)?;

    ASSET_WHITELIST.save(deps.storage, &normalized_asset_whitelist)?;

    Ok(vec![])
}

fn update_nft_whitelist(deps: DepsMut, msg: UpdateNftWhitelistMsg) -> DaoResult<Vec<SubMsg>> {
    for add in msg.add {
        NFT_WHITELIST.save(deps.storage, deps.api.addr_validate(add.as_ref())?, &())?;
    }
    for remove in msg.remove {
        NFT_WHITELIST.remove(deps.storage, deps.api.addr_validate(remove.as_ref())?);
    }

    Ok(vec![])
}

fn upgrade_dao(env: Env, msg: UpgradeDaoMsg) -> DaoResult<Vec<SubMsg>> {
    Ok(vec![SubMsg::new(WasmMsg::Migrate {
        contract_addr: env.contract.address.to_string(),
        new_code_id: msg.new_dao_code_id,
        msg: msg.migrate_msg,
    })])
}

fn execute_msgs(msg: ExecuteMsgsMsg) -> DaoResult<Vec<SubMsg>> {
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
    env: Env,
    msg: ModifyMultisigMembershipMsg,
) -> DaoResult<Vec<SubMsg>> {
    validate_modify_multisig_membership(deps.as_ref(), &msg)?;

    let mut total_weight = load_total_multisig_weight(deps.storage)?;

    let mut submsgs = vec![];

    let mut new_user_weights: Vec<UserWeight> = vec![];

    for edit_member in msg.edit_members {
        let member_addr = deps.api.addr_validate(&edit_member.address)?;

        new_user_weights.push(UserWeight {
            user: edit_member.address,
            weight: edit_member.weight,
        });

        let old_member_weight = MULTISIG_MEMBERS
            .may_load(deps.storage, member_addr.clone())?
            .unwrap_or_default();

        if edit_member.weight == Uint128::zero() {
            MULTISIG_MEMBERS.remove(deps.storage, member_addr.clone())
        } else {
            MULTISIG_MEMBERS.save(deps.storage, member_addr.clone(), &edit_member.weight)?
        }

        if old_member_weight != edit_member.weight {
            submsgs.push(update_user_votes(
                deps.as_ref(),
                member_addr,
                edit_member.weight,
            )?);

            total_weight = if old_member_weight > edit_member.weight {
                total_weight.sub(old_member_weight.sub(edit_member.weight))
            } else {
                total_weight.add(edit_member.weight.sub(old_member_weight))
            }
        }
    }

    save_total_multisig_weight(deps.storage, total_weight, &env.block)?;

    let funds_distributor = FUNDS_DISTRIBUTOR_CONTRACT.load(deps.storage)?;

    submsgs.push(SubMsg::new(wasm_execute(
        funds_distributor.to_string(),
        &funds_distributor_api::msg::ExecuteMsg::UpdateUserWeights(UpdateUserWeightsMsg {
            new_user_weights,
        }),
        vec![],
    )?));

    Ok(submsgs)
}

// TODO: tests
fn distribute_funds(ctx: &mut Context, msg: DistributeFundsMsg) -> DaoResult<Vec<SubMsg>> {
    let mut native_funds: Vec<Coin> = vec![];
    let mut submsgs: Vec<SubMsg> = vec![];

    let funds_distributor = FUNDS_DISTRIBUTOR_CONTRACT.load(ctx.deps.storage)?;

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
) -> DaoResult<Vec<SubMsg>> {
    let funds_distributor = FUNDS_DISTRIBUTOR_CONTRACT.load(ctx.deps.storage)?;

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

pub fn receive_cw20(ctx: &mut Context, cw20_msg: Cw20ReceiveMsg) -> DaoResult<Response> {
    // only membership CW20 contract can execute this message
    let dao_type = DAO_TYPE.load(ctx.deps.storage)?;
    let membership_contract = DAO_MEMBERSHIP_CONTRACT.load(ctx.deps.storage)?;
    if dao_type != Token || ctx.info.sender != membership_contract {
        return Err(InvalidStakingAsset);
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
            CW20_STAKES.save(ctx.deps.storage, sender.clone(), &new_stake)?;

            let update_funds_distributor_submsg = update_funds_distributor(ctx, sender, new_stake)?;

            Ok(Response::new()
                .add_attribute("action", "stake_cw20")
                .add_attribute("total_staked", new_total_staked.to_string())
                .add_attribute("stake", new_stake.to_string())
                .add_submessage(update_funds_distributor_submsg))
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

pub fn receive_cw721(ctx: &mut Context, cw721_msg: ReceiveNftMsg) -> DaoResult<Response> {
    // only membership CW721 contract can execute this message
    let dao_type = DAO_TYPE.load(ctx.deps.storage)?;
    let membership_contract = DAO_MEMBERSHIP_CONTRACT.load(ctx.deps.storage)?;
    if dao_type != Nft || ctx.info.sender != membership_contract {
        return Err(InvalidStakingAsset);
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

            let nft_stake = NftStake {
                staker: staker.clone(),
                token_id,
            };

            save_nft_stake(ctx.deps.storage, &nft_stake)?;

            let qctx = QueryContext {
                deps: ctx.deps.as_ref(),
                env: ctx.env.clone(),
            };

            let new_user_stake = get_user_staked_nfts(qctx, staker.clone())?.amount;

            let update_funds_distributor_submsg =
                update_funds_distributor(ctx, staker, new_user_stake)?;

            Ok(Response::new()
                .add_attribute("action", "stake_cw721")
                .add_attribute("total_staked", new_total_staked.to_string())
                .add_submessage(update_funds_distributor_submsg))
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
            if dao_type != Token {
                return Err(InvalidStakingAsset);
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

            let release_at = calculate_release_at(ctx)?;

            add_claim(
                ctx.deps.storage,
                &ctx.info.sender,
                Claim {
                    asset: Cw20(Cw20ClaimAsset { amount: msg.amount }),
                    release_at,
                },
            )?;

            let update_user_votes_submsg =
                update_user_votes(ctx.deps.as_ref(), ctx.info.sender.clone(), new_stake)?;

            let update_funds_distributor_submsg =
                update_funds_distributor(ctx, ctx.info.sender.clone(), new_stake)?;

            Ok(Response::new()
                .add_attribute("action", "unstake_cw20")
                .add_attribute("total_staked", new_total_staked.to_string())
                .add_attribute("stake", new_stake.to_string())
                .add_submessage(update_user_votes_submsg)
                .add_submessage(update_funds_distributor_submsg))
        }
        UnstakeMsg::Cw721(msg) => {
            if dao_type != Nft {
                return Err(InvalidStakingAsset);
            }

            for token in &msg.tokens {
                // TODO: might be too slow, can we load this in a batch?
                let nft_stake = NFT_STAKES().may_load(ctx.deps.storage, token.to_string())?;

                match nft_stake {
                    None => {
                        return Err(NoNftTokenStaked {
                            token_id: token.to_string(),
                        });
                    }
                    Some(stake) => {
                        if stake.staker != ctx.info.sender {
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

            let release_at = calculate_release_at(ctx)?;

            add_claim(
                ctx.deps.storage,
                &ctx.info.sender,
                Claim {
                    asset: Cw721(Cw721ClaimAsset { tokens: msg.tokens }),
                    release_at,
                },
            )?;

            let qctx = QueryContext {
                deps: ctx.deps.as_ref(),
                env: ctx.env.clone(),
            };
            let new_user_stake = get_user_staked_nfts(qctx, ctx.info.sender.clone())?.amount;

            let update_user_votes_submsg =
                update_user_votes(ctx.deps.as_ref(), ctx.info.sender.clone(), new_user_stake)?;

            let update_funds_distributor_submsg =
                update_funds_distributor(ctx, ctx.info.sender.clone(), new_user_stake)?;

            Ok(Response::new()
                .add_attribute("action", "unstake_cw721")
                .add_attribute("total_staked", new_total_staked.to_string())
                .add_submessage(update_user_votes_submsg)
                .add_submessage(update_funds_distributor_submsg))
        }
    }
}

fn calculate_release_at(ctx: &mut Context) -> DaoResult<ReleaseAt> {
    let gov_config = DAO_GOV_CONFIG.load(ctx.deps.storage)?;

    let release_at = match gov_config.unlocking_period {
        Height(height) => ReleaseAt::Height((ctx.env.block.height + height).into()),
        Time(time) => ReleaseAt::Timestamp(ctx.env.block.time.plus_seconds(time)),
    };
    Ok(release_at)
}

pub fn update_user_votes(deps: Deps, user: Addr, new_amount: Uint128) -> DaoResult<SubMsg> {
    let governance_contract = ENTERPRISE_GOVERNANCE_CONTRACT.load(deps.storage)?;

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

fn update_funds_distributor(
    ctx: &mut Context,
    user: Addr,
    new_user_stake: Uint128,
) -> DaoResult<SubMsg> {
    let funds_distributor = FUNDS_DISTRIBUTOR_CONTRACT.load(ctx.deps.storage)?;

    let update_submsg = SubMsg::new(wasm_execute(
        funds_distributor.to_string(),
        &funds_distributor_api::msg::ExecuteMsg::UpdateUserWeights(UpdateUserWeightsMsg {
            new_user_weights: vec![UserWeight {
                user: user.to_string(),
                weight: new_user_stake,
            }],
        }),
        vec![],
    )?);
    Ok(update_submsg)
}

pub fn claim(ctx: &mut Context) -> DaoResult<Response> {
    let claims = CLAIMS
        .may_load(ctx.deps.storage, &ctx.info.sender)?
        .unwrap_or_default();

    if claims.is_empty() {
        return Err(NothingToClaim);
    }

    let block = ctx.env.block.clone();

    // TODO: this is real brute force, when indexed map is in we should filter smaller data set
    let mut releasable_claims: Vec<Claim> = vec![];
    let remaining_claims = claims
        .into_iter()
        .filter_map(|claim| {
            let is_releasable = is_releasable(&claim, &block);
            if is_releasable {
                releasable_claims.push(claim);
                None
            } else {
                Some(claim)
            }
        })
        .collect();

    if releasable_claims.is_empty() {
        return Err(NothingToClaim);
    }

    CLAIMS.save(ctx.deps.storage, &ctx.info.sender, &remaining_claims)?;

    let dao_membership_contract = DAO_MEMBERSHIP_CONTRACT.load(ctx.deps.storage)?;

    let mut submsgs: Vec<SubMsg> = vec![];
    for releasable_claim in releasable_claims {
        match releasable_claim.asset {
            Cw20(msg) => submsgs.push(SubMsg::new(
                Asset::cw20(dao_membership_contract.clone(), msg.amount)
                    .transfer_msg(ctx.info.sender.clone())?,
            )),
            Cw721(msg) => {
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
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> DaoResult<Response> {
    match msg.id {
        DAO_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let addr = deps.api.addr_validate(&contract_address)?;

            DAO_MEMBERSHIP_CONTRACT.save(deps.storage, &addr)?;

            Ok(Response::new())
        }
        ENTERPRISE_GOVERNANCE_CONTRACT_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let addr = deps.api.addr_validate(&contract_address)?;

            ENTERPRISE_GOVERNANCE_CONTRACT.save(deps.storage, &addr)?;

            Ok(Response::new().add_attribute("governance_contract", addr.to_string()))
        }
        FUNDS_DISTRIBUTOR_CONTRACT_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;

            let addr = deps.api.addr_validate(&contract_address)?;

            FUNDS_DISTRIBUTOR_CONTRACT.save(deps.storage, &addr)?;

            Ok(Response::new().add_attribute("funds_distributor_contract", addr.to_string()))
        }
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
        _ => Err(Std(StdError::generic_err("No such reply ID found"))),
    }
}

fn parse_poll_id(msg: Reply) -> DaoResult<PollId> {
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
    let dao_council = DAO_COUNCIL.load(qctx.deps.storage)?;
    let dao_type = DAO_TYPE.load(qctx.deps.storage)?;
    let dao_membership_contract = DAO_MEMBERSHIP_CONTRACT.load(qctx.deps.storage)?;
    let enterprise_factory_contract = ENTERPRISE_FACTORY_CONTRACT.load(qctx.deps.storage)?;
    let funds_distributor_contract = FUNDS_DISTRIBUTOR_CONTRACT.load(qctx.deps.storage)?;
    let dao_code_version = DAO_CODE_VERSION.load(qctx.deps.storage)?;

    Ok(DaoInfoResponse {
        creation_date,
        metadata,
        gov_config,
        dao_council,
        dao_type,
        dao_membership_contract,
        enterprise_factory_contract,
        funds_distributor_contract,
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
        .collect();

    Ok(NftWhitelistResponse { nfts })
}

pub fn query_proposal(qctx: QueryContext, msg: ProposalParams) -> DaoResult<ProposalResponse> {
    let poll = query_poll(&qctx, msg.proposal_id)?;

    let proposal = poll_to_proposal_response(qctx.deps, &qctx.env, &poll.poll)?;

    Ok(proposal)
}

fn query_poll(qctx: &QueryContext, poll_id: PollId) -> DaoResult<PollResponse> {
    let governance_contract = ENTERPRISE_GOVERNANCE_CONTRACT.load(qctx.deps.storage)?;

    let poll: PollResponse = qctx.deps.querier.query_wasm_smart(
        governance_contract.to_string(),
        &enterprise_governance_api::msg::QueryMsg::Poll(PollParams { poll_id }),
    )?;
    Ok(poll)
}

pub fn query_proposals(qctx: QueryContext, msg: ProposalsParams) -> DaoResult<ProposalsResponse> {
    let governance_contract = ENTERPRISE_GOVERNANCE_CONTRACT.load(qctx.deps.storage)?;

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

fn query_poll_status(qctx: &QueryContext, poll_id: PollId) -> DaoResult<PollStatusResponse> {
    let governance_contract = ENTERPRISE_GOVERNANCE_CONTRACT.load(qctx.deps.storage)?;
    let poll_status_response: PollStatusResponse = qctx.deps.querier.query_wasm_smart(
        governance_contract.to_string(),
        &enterprise_governance_api::msg::QueryMsg::PollStatus { poll_id },
    )?;

    Ok(poll_status_response)
}

fn poll_to_proposal_response(deps: Deps, env: &Env, poll: &Poll) -> DaoResult<ProposalResponse> {
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
    dao_type: DaoType,
    store: &dyn Storage,
    height: u64,
) -> StdResult<Uint128> {
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

pub fn query_member_vote(
    qctx: QueryContext,
    params: MemberVoteParams,
) -> DaoResult<MemberVoteResponse> {
    let governance_contract = ENTERPRISE_GOVERNANCE_CONTRACT.load(qctx.deps.storage)?;
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
) -> DaoResult<ProposalVotesResponse> {
    let governance_contract = ENTERPRISE_GOVERNANCE_CONTRACT.load(qctx.deps.storage)?;
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

pub fn query_member_info(
    qctx: QueryContext,
    msg: QueryMemberInfoMsg,
) -> DaoResult<MemberInfoResponse> {
    let dao_type = DAO_TYPE.load(qctx.deps.storage)?;

    let voting_power = calculate_member_voting_power(qctx, msg.member_address, dao_type)?;

    Ok(MemberInfoResponse { voting_power })
}

fn calculate_member_voting_power(
    qctx: QueryContext,
    member: String,
    dao_type: DaoType,
) -> DaoResult<Decimal> {
    let total_weight = load_total_weight(qctx.deps, dao_type.clone())?;

    if total_weight == Uint128::zero() {
        Ok(Decimal::zero())
    } else {
        let member = qctx.deps.api.addr_validate(&member)?;
        let member_weight = load_member_weight(qctx.deps, member, dao_type)?;

        Ok(Decimal::from_ratio(member_weight, total_weight))
    }
}

fn load_member_weight(deps: Deps, member: Addr, dao_type: DaoType) -> DaoResult<Uint128> {
    let member_weight = match dao_type {
        Token => CW20_STAKES
            .may_load(deps.storage, member)?
            .unwrap_or_default(),
        Nft => (load_all_nft_stakes_for_user(deps.storage, member)?.len() as u128).into(),
        Multisig => MULTISIG_MEMBERS
            .may_load(deps.storage, member)?
            .unwrap_or_default(),
    };

    Ok(member_weight)
}

fn load_total_weight(deps: Deps, dao_type: DaoType) -> DaoResult<Uint128> {
    let total_weight = match dao_type {
        Token | Nft => load_total_staked(deps.storage)?,
        Multisig => load_total_multisig_weight(deps.storage)?,
    };

    Ok(total_weight)
}

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
                .unwrap_or(DEFAULT_QUERY_LIMIT as u32)
                .min(MAX_QUERY_LIMIT as u32) as usize,
        )
        .collect::<StdResult<Vec<(Addr, Uint128)>>>()?
        .into_iter()
        .map(|(addr, weight)| MultisigMember {
            address: addr.to_string(),
            weight,
        })
        .collect();

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
        Multisig => UserStake::None,
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
    let staked_tokens: Vec<NftTokenId> = load_all_nft_stakes_for_user(qctx.deps.storage, user)?
        .into_iter()
        .map(|stake| stake.token_id)
        .collect();
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
        .collect();

    Ok(ClaimsResponse {
        claims: releasable_claims,
    })
}

pub fn query_cw20_treasury(qctx: QueryContext) -> DaoResult<AssetTreasuryResponse> {
    let enterprise_factory = ENTERPRISE_FACTORY_CONTRACT.load(qctx.deps.storage)?;
    let global_asset_whitelist: AssetWhitelistResponse = qctx
        .deps
        .querier
        .query_wasm_smart(enterprise_factory, &GlobalAssetWhitelist {})?;

    let mut asset_whitelist = query_asset_whitelist(qctx.clone())?.assets;

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

fn query_nft_treasury(qctx: QueryContext) -> DaoResult<NftTreasuryResponse> {
    let enterprise_factory = ENTERPRISE_FACTORY_CONTRACT.load(qctx.deps.storage)?;
    let global_nft_whitelist: NftWhitelistResponse = qctx
        .deps
        .querier
        .query_wasm_smart(enterprise_factory, &GlobalNftWhitelist {})?;

    let mut nft_whitelist = query_nft_whitelist(qctx.clone())?.nfts;

    for nft in global_nft_whitelist.nfts {
        // TODO: suboptimal, use maps or sth
        if !nft_whitelist.contains(&nft) {
            nft_whitelist.push(nft);
        }
    }

    let dao_membership_contract = DAO_MEMBERSHIP_CONTRACT.load(qctx.deps.storage)?;

    let dao_address = qctx.env.contract.address.as_ref();

    let mut nfts = nft_whitelist
        .into_iter()
        // if the DAO has an NFT membership, remove it from the whitelist, as it is handled separately
        .filter(|nft| nft != &dao_membership_contract)
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
            .collect();

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
    let query_owner_tokens = Cw721QueryMsg::Tokens {
        owner: owner.to_string(),
        start_after: start_after.cloned(),
        limit: Some(u32::MAX),
    };

    let talis_friendly_tokens_response: TalisFriendlyTokensResponse = qctx
        .deps
        .querier
        .query_wasm_smart(nft_addr.to_string(), &query_owner_tokens)?;

    Ok(talis_friendly_tokens_response.to_tokens_response()?)
}

fn is_releasable(claim: &Claim, block_info: &BlockInfo) -> bool {
    match claim.release_at {
        ReleaseAt::Timestamp(timestamp) => block_info.time >= timestamp,
        ReleaseAt::Height(height) => block_info.height >= height.u64(),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> DaoResult<Response> {
    let contract_version = get_contract_version(deps.storage)?;

    let mut submsgs: Vec<SubMsg> = vec![];

    if contract_version.version == "0.3.0" {
        let funds_distributor = FUNDS_DISTRIBUTOR_CONTRACT.load(deps.storage)?;

        submsgs.push(SubMsg::new(WasmMsg::Migrate {
            contract_addr: funds_distributor.to_string(),
            new_code_id: msg.funds_distributor_code_id,
            msg: to_binary(&funds_distributor_api::msg::MigrateMsg {
                minimum_eligible_weight: msg.minimum_eligible_weight,
            })?,
        }));
    }

    DAO_CODE_VERSION.save(deps.storage, &Uint64::from(CODE_VERSION))?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("action", "migrate")
        .add_submessages(submsgs))
}
