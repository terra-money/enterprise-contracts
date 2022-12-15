use crate::contract::execute;
use crate::tests::helpers::{
    assert_proposal_result_amount, assert_total_stake, assert_user_nft_stake,
    assert_user_token_stake, create_stub_proposal, existing_nft_dao_membership,
    existing_token_dao_membership, instantiate_stub_dao, stake_nfts, stake_tokens,
    stub_dao_membership_info, stub_token_info, unstake_nfts, unstake_tokens, CW20_ADDR, NFT_ADDR,
};
use crate::tests::querier::mock_querier::mock_dependencies;
use common::cw::testing::{mock_env, mock_info, mock_query_ctx};
use enterprise_protocol::api::CastVoteMsg;
use enterprise_protocol::api::DaoType::Multisig;
use enterprise_protocol::error::DaoResult;
use enterprise_protocol::msg::ExecuteMsg;
use poll_engine::api::DefaultVoteOption;

#[test]
fn unstake_token_dao() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[("cw20_addr", &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
    )?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender", 50u8)?;

    // cannot unstake CW721 in token DAO
    let result = unstake_nfts(deps.as_mut(), &env, "sender", vec!["token1"]);
    assert!(result.is_err());

    // cannot unstake more than staked
    let result = unstake_tokens(deps.as_mut(), &env, "sender", 51u8);
    assert!(result.is_err());

    unstake_tokens(deps.as_mut(), &env, "sender", 14u8)?;
    unstake_tokens(deps.as_mut(), &env, "sender", 15u8)?;

    assert_user_token_stake(mock_query_ctx(deps.as_ref(), &env), "sender", 21u8);
    assert_total_stake(mock_query_ctx(deps.as_ref(), &env), 21u8);

    Ok(())
}

#[test]
fn unstake_nft_dao() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier.with_num_tokens(&[(NFT_ADDR, 100u64)]);

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
        vec!["token1", "token2", "token3"],
    )?;

    // cannot unstake CW20 in NFT DAO
    let result = unstake_tokens(deps.as_mut(), &env, "sender", 1u8);
    assert!(result.is_err());

    // cannot unstake if they don't have it staked
    let result = unstake_nfts(deps.as_mut(), &env, "sender", vec!["token4"]);
    assert!(result.is_err());

    unstake_nfts(deps.as_mut(), &env, "sender", vec!["token1"])?;
    unstake_nfts(deps.as_mut(), &env, "sender", vec!["token3"])?;

    assert_user_nft_stake(
        mock_query_ctx(deps.as_ref(), &env),
        "sender",
        vec!["token2".to_string()],
    );
    assert_total_stake(mock_query_ctx(deps.as_ref(), &env), 1u8);

    Ok(())
}

// TODO: probably has to be replaced by an integration test and deleted
#[ignore]
#[test]
fn unstake_multisig_dao() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_multisig_members(&[("cw3_addr", &[("sender", 10u64)])]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        stub_dao_membership_info(Multisig, "cw3_addr"),
        None,
    )?;

    // cannot unstake in multisig DAO
    let result = unstake_tokens(deps.as_mut(), &env, "sender", 1u8);
    assert!(result.is_err());

    // cannot unstake in multisig DAO
    let result = unstake_nfts(deps.as_mut(), &env, "sender", vec!["token1"]);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn unstaking_tokens_reduces_existing_votes() -> DaoResult<()> {
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

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user", 100u8)?;

    create_stub_proposal(deps.as_mut(), &env, &info)?;

    // TODO: extract to a helper
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("user", &vec![]),
        ExecuteMsg::CastVote(CastVoteMsg {
            proposal_id: 1,
            outcome: DefaultVoteOption::No,
        }),
    )?;

    unstake_tokens(deps.as_mut(), &env, "user", 60u8)?;
    assert_proposal_result_amount(
        &mock_query_ctx(deps.as_ref(), &env),
        1,
        DefaultVoteOption::No,
        40u128,
    );

    unstake_tokens(deps.as_mut(), &env, "user", 30u8)?;
    assert_proposal_result_amount(
        &mock_query_ctx(deps.as_ref(), &env),
        1,
        DefaultVoteOption::No,
        10u128,
    );

    unstake_tokens(deps.as_mut(), &env, "user", 10u8)?;
    assert_proposal_result_amount(
        &mock_query_ctx(deps.as_ref(), &env),
        1,
        DefaultVoteOption::No,
        0u128,
    );

    Ok(())
}

#[test]
fn unstaking_nfts_reduces_existing_votes() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier.with_num_tokens(&[(NFT_ADDR, 100u64)]);

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
        "user",
        vec!["token1", "token2", "token3"],
    )?;

    create_stub_proposal(deps.as_mut(), &env, &mock_info("user", &vec![]))?;

    // TODO: extract to a helper
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("user", &vec![]),
        ExecuteMsg::CastVote(CastVoteMsg {
            proposal_id: 1,
            outcome: DefaultVoteOption::Yes,
        }),
    )?;

    unstake_nfts(deps.as_mut(), &env, "user", vec!["token2"])?;
    assert_proposal_result_amount(
        &mock_query_ctx(deps.as_ref(), &env),
        1,
        DefaultVoteOption::Yes,
        2u128,
    );

    unstake_nfts(deps.as_mut(), &env, "user", vec!["token3"])?;
    assert_proposal_result_amount(
        &mock_query_ctx(deps.as_ref(), &env),
        1,
        DefaultVoteOption::Yes,
        1u128,
    );

    unstake_nfts(deps.as_mut(), &env, "user", vec!["token1"])?;
    assert_proposal_result_amount(
        &mock_query_ctx(deps.as_ref(), &env),
        1,
        DefaultVoteOption::Yes,
        0u128,
    );

    Ok(())
}

// TODO: unstaking doesn't reduce votes in ended proposals!
