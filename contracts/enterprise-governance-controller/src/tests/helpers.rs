use crate::contract::{
    execute, instantiate, query_member_info, query_proposal, query_total_staked_amount,
    query_user_stake, reply, ENTERPRISE_GOVERNANCE_CONTRACT_INSTANTIATE_REPLY_ID,
    FUNDS_DISTRIBUTOR_CONTRACT_INSTANTIATE_REPLY_ID,
};
use common::cw::testing::{mock_info, mock_query_ctx};
use common::cw::QueryContext;
use cosmwasm_std::{
    to_binary, Binary, Decimal, DepsMut, Env, MessageInfo, Reply, Response, SubMsgResponse,
    SubMsgResult, Uint128,
};
use cw20::Cw20ReceiveMsg;
use cw20_base::state::TokenInfo;
use cw_utils::Duration;
use enterprise_protocol::api::DaoMembershipInfo::{Existing, New};
use enterprise_protocol::api::DaoType::{Multisig, Nft, Token};
use enterprise_protocol::api::{
    CastVoteMsg, CreateProposalMsg, DaoCouncilSpec, DaoGovConfig, DaoMembershipInfo, DaoMetadata,
    DaoSocialData, DaoType, ExistingDaoMembershipMsg, Logo, MultisigMember, NewDaoMembershipMsg,
    NewMembershipInfo, NewMultisigMembershipInfo, NftUserStake, ProposalAction, ProposalId,
    ProposalParams, ProposalStatus, QueryMemberInfoMsg, ReceiveNftMsg, TokenUserStake,
    UnstakeCw20Msg, UnstakeCw721Msg, UnstakeMsg, UserStake, UserStakeParams,
};
use enterprise_protocol::error::DaoResult;
use enterprise_protocol::msg::ExecuteMsg::{
    CastCouncilVote, CastVote, CreateCouncilProposal, CreateProposal,
};
use enterprise_protocol::msg::{Cw20HookMsg, ExecuteMsg, InstantiateMsg};
use itertools::Itertools;
use poll_engine_api::api::VoteOutcome;
use std::collections::BTreeMap;
use ExecuteMsg::{Receive, ReceiveNft, Unstake};
use NewMembershipInfo::NewMultisig;

pub const ENTERPRISE_FACTORY_ADDR: &str = "enterprise_factory_addr";
pub const ENTERPRISE_GOVERNANCE_ADDR: &str = "enterprise_governance_addr";
pub const FUNDS_DISTRIBUTOR_ADDR: &str = "funds_distributor_addr";

pub const ENTERPRISE_GOVERNANCE_CODE_ID: u64 = 301;
pub const FUNDS_DISTRIBUTOR_CODE_ID: u64 = 302;

pub const CW20_ADDR: &str = "cw20_addr";
pub const NFT_ADDR: &str = "cw721_addr";

pub const DAO_ADDR: &str = "dao_contract_address";

pub const PROPOSAL_TITLE: &str = "Proposal title";
pub const PROPOSAL_DESCRIPTION: &str = "Description";

pub fn stub_dao_metadata() -> DaoMetadata {
    DaoMetadata {
        name: "Stub DAO".to_string(),
        description: None,
        logo: Logo::None,
        socials: DaoSocialData {
            github_username: None,
            discord_username: None,
            twitter_username: None,
            telegram_username: None,
        },
    }
}

pub fn stub_dao_gov_config() -> DaoGovConfig {
    DaoGovConfig {
        quorum: Decimal::from_ratio(1u8, 10u8),
        threshold: Decimal::from_ratio(3u8, 10u8),
        veto_threshold: None,
        vote_duration: 1,
        unlocking_period: Duration::Height(100),
        minimum_deposit: None,
        allow_early_proposal_execution: false,
    }
}

pub fn stub_token_dao_membership_info() -> DaoMembershipInfo {
    existing_token_dao_membership("addr")
}

pub fn existing_token_dao_membership(addr: &str) -> DaoMembershipInfo {
    stub_dao_membership_info(Token, addr)
}

pub fn stub_token_info() -> TokenInfo {
    TokenInfo {
        name: "some_name".to_string(),
        symbol: "SMBL".to_string(),
        decimals: 6,
        total_supply: Uint128::from(1000u16),
        mint: None,
    }
}

pub fn stub_enterprise_factory_contract() -> String {
    "enterprise_factory_addr".to_string()
}

pub fn stub_nft_dao_membership_info() -> DaoMembershipInfo {
    stub_dao_membership_info(Nft, "addr")
}

pub fn existing_nft_dao_membership(addr: &str) -> DaoMembershipInfo {
    stub_dao_membership_info(Nft, addr)
}

pub fn stub_multisig_dao_membership_info() -> DaoMembershipInfo {
    stub_dao_membership_info(Multisig, "addr")
}

pub fn multisig_dao_membership_info_with_members(members: &[(&str, u64)]) -> DaoMembershipInfo {
    let multisig_members = members
        .into_iter()
        .map(|(addr, weight)| MultisigMember {
            address: addr.to_string(),
            weight: (*weight).into(),
        })
        .collect_vec();
    New(NewDaoMembershipMsg {
        membership_contract_code_id: 0,
        membership_info: NewMultisig(NewMultisigMembershipInfo { multisig_members }),
    })
}

pub fn stub_dao_membership_info(dao_type: DaoType, addr: &str) -> DaoMembershipInfo {
    Existing(ExistingDaoMembershipMsg {
        dao_type,
        membership_contract_addr: addr.to_string(),
    })
}

pub fn instantiate_stub_dao(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    membership: DaoMembershipInfo,
    gov_config: Option<DaoGovConfig>,
    dao_council: Option<DaoCouncilSpec>,
) -> DaoResult<Response> {
    let response = instantiate(
        deps.branch(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
            funds_distributor_code_id: FUNDS_DISTRIBUTOR_CODE_ID,
            dao_metadata: stub_dao_metadata(),
            dao_gov_config: gov_config.unwrap_or(stub_dao_gov_config()),
            dao_council,
            dao_membership_info: membership,
            enterprise_factory_contract: ENTERPRISE_FACTORY_ADDR.to_string(),
            asset_whitelist: None,
            nft_whitelist: None,
            minimum_weight_for_rewards: None,
        },
    )?;

    reply_default_instantiate_data(deps, env.clone())?;

    Ok(response)
}

pub fn stake_tokens(
    deps: DepsMut,
    env: &Env,
    cw20_contract: &str,
    user: &str,
    amount: impl Into<Uint128>,
) -> DaoResult<()> {
    execute(
        deps,
        env.clone(),
        mock_info(cw20_contract, &vec![]),
        Receive(Cw20ReceiveMsg {
            sender: user.to_string(),
            amount: amount.into(),
            msg: to_binary(&Cw20HookMsg::Stake {})?,
        }),
    )?;

    Ok(())
}

pub fn stake_nfts(
    deps: &mut DepsMut,
    env: &Env,
    nft_contract: &str,
    user: &str,
    tokens: Vec<impl Into<String>>,
) -> DaoResult<()> {
    for token in tokens {
        execute(
            deps.branch(),
            env.clone(),
            mock_info(nft_contract, &vec![]),
            ReceiveNft(ReceiveNftMsg {
                edition: None,
                sender: user.to_string(),
                token_id: token.into(),
                msg: to_binary(&Cw20HookMsg::Stake {})?,
            }),
        )?;
    }

    Ok(())
}

pub fn unstake_tokens(
    deps: DepsMut,
    env: &Env,
    user: &str,
    amount: impl Into<Uint128>,
) -> DaoResult<()> {
    execute(
        deps,
        env.clone(),
        mock_info(user, &vec![]),
        Unstake(UnstakeMsg::Cw20(UnstakeCw20Msg {
            amount: amount.into(),
        })),
    )?;

    Ok(())
}

pub fn unstake_nfts(
    deps: DepsMut,
    env: &Env,
    user: &str,
    tokens: Vec<impl Into<String>>,
) -> DaoResult<()> {
    let tokens = tokens.into_iter().map(|token| token.into()).collect_vec();
    execute(
        deps,
        env.clone(),
        mock_info(user, &vec![]),
        Unstake(UnstakeMsg::Cw721(UnstakeCw721Msg { tokens })),
    )?;

    Ok(())
}

pub fn create_stub_proposal(deps: DepsMut, env: &Env, info: &MessageInfo) -> DaoResult<Response> {
    execute(
        deps,
        env.clone(),
        info.clone(),
        CreateProposal(CreateProposalMsg {
            title: "Proposal title".to_string(),
            description: None,
            proposal_actions: vec![],
        }),
    )
}

pub fn create_proposal(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    title: Option<&str>,
    description: Option<&str>,
    proposal_actions: Vec<ProposalAction>,
) -> DaoResult<Response> {
    execute(
        deps,
        env.clone(),
        info.clone(),
        CreateProposal(CreateProposalMsg {
            title: title.unwrap_or(PROPOSAL_TITLE).to_string(),
            description: description.map(|desc| desc.to_string()),
            proposal_actions,
        }),
    )
}

pub fn create_council_proposal(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    title: Option<&str>,
    description: Option<&str>,
    proposal_actions: Vec<ProposalAction>,
) -> DaoResult<Response> {
    execute(
        deps,
        env.clone(),
        info.clone(),
        CreateCouncilProposal(CreateProposalMsg {
            title: title.unwrap_or(PROPOSAL_TITLE).to_string(),
            description: description.map(|desc| desc.to_string()),
            proposal_actions,
        }),
    )
}

pub fn vote_on_proposal(
    deps: DepsMut,
    env: &Env,
    voter: &str,
    proposal_id: ProposalId,
    outcome: VoteOutcome,
) -> DaoResult<Response> {
    execute(
        deps,
        env.clone(),
        mock_info(voter, &vec![]),
        CastVote(CastVoteMsg {
            proposal_id,
            outcome,
        }),
    )
}

pub fn vote_on_council_proposal(
    deps: DepsMut,
    env: &Env,
    voter: &str,
    proposal_id: ProposalId,
    outcome: VoteOutcome,
) -> DaoResult<Response> {
    execute(
        deps,
        env.clone(),
        mock_info(voter, &vec![]),
        CastCouncilVote(CastVoteMsg {
            proposal_id,
            outcome,
        }),
    )
}

pub fn assert_user_token_stake(qctx: QueryContext, user: &str, amount: impl Into<Uint128>) {
    let user_stake = query_user_stake(
        qctx,
        UserStakeParams {
            user: user.to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        user_stake.user_stake,
        UserStake::Token(TokenUserStake {
            amount: amount.into(),
        })
    );
}

pub fn assert_user_stake_is_none(qctx: QueryContext, user: &str) {
    let user_stake = query_user_stake(
        qctx,
        UserStakeParams {
            user: user.to_string(),
        },
    )
    .unwrap();
    assert_eq!(user_stake.user_stake, UserStake::None,);
}

pub fn assert_user_nft_stake(qctx: QueryContext, user: &str, tokens: Vec<String>) {
    let user_stake = query_user_stake(
        qctx,
        UserStakeParams {
            user: user.to_string(),
        },
    )
    .unwrap();
    let amount = tokens.len() as u128;
    assert_eq!(
        user_stake.user_stake,
        UserStake::Nft(NftUserStake {
            tokens,
            amount: amount.into(),
        })
    );
}

pub fn assert_total_stake(qctx: QueryContext, amount: impl Into<Uint128>) {
    let total_stake = query_total_staked_amount(qctx).unwrap();
    assert_eq!(total_stake.total_staked_amount, amount.into());
}

pub fn assert_member_voting_power(qctx: &QueryContext, member: &str, voting_power: Decimal) {
    let qctx = mock_query_ctx(qctx.deps, &qctx.env);
    let member_info = query_member_info(
        qctx,
        QueryMemberInfoMsg {
            member_address: member.to_string(),
        },
    )
    .unwrap();
    assert_eq!(member_info.voting_power, voting_power);
}

pub fn assert_proposal_status(
    qctx: &QueryContext,
    proposal_id: ProposalId,
    status: ProposalStatus,
) {
    let qctx = QueryContext::from(qctx.deps, qctx.env.clone());
    let proposal = query_proposal(qctx, ProposalParams { proposal_id }).unwrap();
    assert_eq!(proposal.proposal.status, status);
}

pub fn assert_proposal_result_amount(
    qctx: &QueryContext,
    proposal_id: ProposalId,
    result: VoteOutcome,
    amount: u128,
) {
    let qctx = QueryContext::from(qctx.deps, qctx.env.clone());
    let proposal = query_proposal(qctx, ProposalParams { proposal_id }).unwrap();
    assert_eq!(proposal.results.get(&(result as u8)), Some(&amount));
}

pub fn assert_proposal_no_votes(qctx: &QueryContext, proposal_id: ProposalId) {
    let qctx = QueryContext::from(qctx.deps, qctx.env.clone());
    let proposal = query_proposal(qctx, ProposalParams { proposal_id }).unwrap();
    assert_eq!(proposal.results, BTreeMap::new());
}

pub fn reply_default_instantiate_data(deps: &mut DepsMut, env: Env) -> DaoResult<()> {
    reply_with_instantiate_data(
        deps.branch(),
        env.clone(),
        FUNDS_DISTRIBUTOR_CONTRACT_INSTANTIATE_REPLY_ID,
        FUNDS_DISTRIBUTOR_ADDR,
    )?;
    reply_with_instantiate_data(
        deps.branch(),
        env.clone(),
        ENTERPRISE_GOVERNANCE_CONTRACT_INSTANTIATE_REPLY_ID,
        ENTERPRISE_GOVERNANCE_ADDR,
    )?;

    Ok(())
}

pub fn reply_with_instantiate_data(
    deps: DepsMut,
    env: Env,
    reply_id: u64,
    addr: &str,
) -> DaoResult<()> {
    reply(
        deps,
        env.clone(),
        Reply {
            id: reply_id,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some(instantiate_addr_reply_data(addr)),
            }),
        },
    )?;

    Ok(())
}

pub fn instantiate_addr_reply_data(addr: &str) -> Binary {
    let mut binary: Vec<u8> = vec![10, addr.len() as u8];

    addr.chars().for_each(|char| binary.push(char as u8));

    binary.into()
}
