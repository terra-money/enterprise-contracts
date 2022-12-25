use crate::contract::execute;
use crate::proposals::ProposalType::General;
use crate::tests::helpers::{
    assert_proposal_no_votes, assert_proposal_result_amount, assert_proposal_status,
    create_stub_proposal, existing_nft_dao_membership, existing_token_dao_membership,
    instantiate_stub_dao, multisig_dao_membership_info_with_members, stake_nfts, stake_tokens,
    stub_dao_gov_config, stub_token_info, vote_on_proposal, CW20_ADDR, NFT_ADDR,
};
use crate::tests::querier::mock_querier::mock_dependencies;
use common::cw::testing::{mock_env, mock_info, mock_query_ctx};
use cosmwasm_std::{Timestamp, Uint128};
use enterprise_protocol::api::{CastVoteMsg, DaoGovConfig, ProposalStatus};
use enterprise_protocol::error::DaoError::Unauthorized;
use enterprise_protocol::error::{DaoError, DaoResult};
use enterprise_protocol::msg::ExecuteMsg::CastVote;
use poll_engine::api::VoteOutcome::{Abstain, No, Veto, Yes};
use poll_engine::error::PollError;
use DaoError::Poll;
use PollError::OutsideVotingPeriod;
use ProposalStatus::InProgress;

#[test]
fn cast_vote_token_dao() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
    )?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender1", 12u8)?;
    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender2", 14u8)?;
    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender3", 7u8)?;

    create_stub_proposal(deps.as_mut(), &env, &info)?;

    let qctx = mock_query_ctx(deps.as_ref(), &env);
    assert_proposal_status(&qctx, 1, General, InProgress);
    assert_proposal_no_votes(&qctx, 1, General);

    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("sender1", &vec![]),
        CastVote(CastVoteMsg {
            proposal_id: 1,
            outcome: Yes,
        }),
    )?;

    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("sender2", &vec![]),
        CastVote(CastVoteMsg {
            proposal_id: 1,
            outcome: Abstain,
        }),
    )?;

    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("sender3", &vec![]),
        CastVote(CastVoteMsg {
            proposal_id: 1,
            outcome: Veto,
        }),
    )?;

    let qctx = mock_query_ctx(deps.as_ref(), &env);
    assert_proposal_status(&qctx, 1, General, InProgress);
    assert_proposal_result_amount(&qctx, 1, General, Yes, 12);
    assert_proposal_result_amount(&qctx, 1, General, Abstain, 14);
    assert_proposal_result_amount(&qctx, 1, General, Veto, 7);

    Ok(())
}

#[test]
fn cast_vote_nft_dao() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier.with_num_tokens(&[(NFT_ADDR, 1000u64)]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_nft_dao_membership(NFT_ADDR),
        None,
    )?;

    stake_nfts(
        &mut deps.as_mut(),
        &env,
        NFT_ADDR,
        "sender",
        vec!["15", "16"],
    )?;

    create_stub_proposal(deps.as_mut(), &env, &info)?;

    let qctx = mock_query_ctx(deps.as_ref(), &env);
    assert_proposal_status(&qctx, 1, General, InProgress);
    assert_proposal_no_votes(&qctx, 1, General);

    execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        CastVote(CastVoteMsg {
            proposal_id: 1,
            outcome: No,
        }),
    )?;

    let qctx = mock_query_ctx(deps.as_ref(), &env);
    assert_proposal_status(&qctx, 1, General, InProgress);
    assert_proposal_result_amount(&qctx, 1, General, No, 2);

    Ok(())
}

#[test]
fn cast_vote_multisig_dao() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        multisig_dao_membership_info_with_members(&[("member1", 1u64), ("member2", 2u64)]),
        None,
    )?;

    create_stub_proposal(deps.as_mut(), &env, &mock_info("member1", &vec![]))?;

    let qctx = mock_query_ctx(deps.as_ref(), &env);
    assert_proposal_status(&qctx, 1, General, InProgress);
    assert_proposal_no_votes(&qctx, 1, General);

    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("member1", &vec![]),
        CastVote(CastVoteMsg {
            proposal_id: 1,
            outcome: No,
        }),
    )?;

    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("member2", &vec![]),
        CastVote(CastVoteMsg {
            proposal_id: 1,
            outcome: Yes,
        }),
    )?;

    let qctx = mock_query_ctx(deps.as_ref(), &env);
    assert_proposal_status(&qctx, 1, General, InProgress);
    assert_proposal_result_amount(&qctx, 1, General, No, 1);
    assert_proposal_result_amount(&qctx, 1, General, Yes, 2);

    Ok(())
}

#[test]
fn cast_vote_by_non_token_holder_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);
    deps.querier
        .with_token_balances(&[(CW20_ADDR, &[("holder", Uint128::one())])]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
    )?;

    create_stub_proposal(deps.as_mut(), &env, &mock_info("holder", &vec![]))?;

    let result = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("non_holder", &vec![]),
        CastVote(CastVoteMsg {
            proposal_id: 1,
            outcome: Yes,
        }),
    );

    assert_eq!(result, Err(Unauthorized));

    Ok(())
}

#[test]
fn cast_vote_by_non_nft_holder_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier.with_num_tokens(&[(NFT_ADDR, 1000u64)]);
    deps.querier
        .with_nft_holders(&[(NFT_ADDR, &[("holder", &["1"])])]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_nft_dao_membership(NFT_ADDR),
        None,
    )?;

    create_stub_proposal(deps.as_mut(), &env, &mock_info("holder", &vec![]))?;

    let result = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("non_holder", &vec![]),
        CastVote(CastVoteMsg {
            proposal_id: 1,
            outcome: Yes,
        }),
    );

    assert_eq!(result, Err(Unauthorized));

    Ok(())
}

#[test]
fn cast_vote_by_non_multisig_member_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        multisig_dao_membership_info_with_members(&[("member1", 1u64)]),
        None,
    )?;

    create_stub_proposal(deps.as_mut(), &env, &mock_info("member1", &vec![]))?;

    let result = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("non_member", &vec![]),
        CastVote(CastVoteMsg {
            proposal_id: 1,
            outcome: Yes,
        }),
    );

    assert_eq!(result, Err(Unauthorized));

    Ok(())
}

#[test]
fn cast_vote_on_expired_proposal_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    let start_time = Timestamp::from_seconds(10000u64);
    let end_time = Timestamp::from_seconds(11000u64);
    env.block.time = start_time;

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);
    deps.querier
        .with_token_balances(&[(CW20_ADDR, &[("sender", Uint128::from(10u8))])]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        Some(DaoGovConfig {
            vote_duration: 1000,
            ..stub_dao_gov_config()
        }),
    )?;

    create_stub_proposal(deps.as_mut(), &env, &info)?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender", 10u8)?;

    env.block.time = end_time;

    let result = vote_on_proposal(deps.as_mut(), &env, "sender", 1, Yes);

    assert_eq!(
        result,
        Err(Poll(OutsideVotingPeriod {
            voting_period: (start_time, end_time),
            now: env.block.time
        }))
    );

    Ok(())
}

#[test]
fn cast_vote_multiple_times_only_records_last_vote() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);
    deps.querier
        .with_token_balances(&[(CW20_ADDR, &[("sender", Uint128::from(10u8))])]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
    )?;

    create_stub_proposal(deps.as_mut(), &env, &info)?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender", 10u8)?;

    vote_on_proposal(deps.as_mut(), &env, "sender", 1, Yes)?;
    vote_on_proposal(deps.as_mut(), &env, "sender", 1, No)?;
    vote_on_proposal(deps.as_mut(), &env, "sender", 1, No)?;

    let qctx = mock_query_ctx(deps.as_ref(), &env);
    assert_proposal_result_amount(&qctx, 1, General, Yes, 0u128);
    assert_proposal_result_amount(&qctx, 1, General, No, 10u128);

    Ok(())
}

#[test]
fn cast_multisig_vote_multiple_times_only_records_last_vote() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        multisig_dao_membership_info_with_members(&[("member", 100u64)]),
        None,
    )?;

    create_stub_proposal(deps.as_mut(), &env, &mock_info("member", &vec![]))?;

    vote_on_proposal(deps.as_mut(), &env, "member", 1, Yes)?;
    vote_on_proposal(deps.as_mut(), &env, "member", 1, No)?;
    vote_on_proposal(deps.as_mut(), &env, "member", 1, No)?;

    let qctx = mock_query_ctx(deps.as_ref(), &env);
    assert_proposal_result_amount(&qctx, 1, General, Yes, 0u128);
    assert_proposal_result_amount(&qctx, 1, General, No, 100u128);

    Ok(())
}
