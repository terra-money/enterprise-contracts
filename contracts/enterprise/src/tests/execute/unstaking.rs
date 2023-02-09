use crate::tests::helpers::{
    assert_proposal_result_amount, assert_total_stake, assert_user_nft_stake,
    assert_user_token_stake, create_stub_proposal, existing_nft_dao_membership,
    existing_token_dao_membership, instantiate_stub_dao, multisig_dao_membership_info_with_members,
    stake_nfts, stake_tokens, stub_token_info, unstake_nfts, unstake_tokens, vote_on_proposal,
    CW20_ADDR, NFT_ADDR,
};
use crate::tests::querier::mock_querier::mock_dependencies;
use common::cw::testing::{mock_env, mock_info, mock_query_ctx};
use enterprise_protocol::api::ProposalType::General;
use enterprise_protocol::error::DaoError::{InvalidStakingAsset, NoNftTokenStaked, Unauthorized};
use enterprise_protocol::error::DaoResult;
use poll_engine_api::api::VoteOutcome::{No, Yes};

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
        None,
    )?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender", 50u8)?;

    // cannot unstake CW721 in token DAO
    let result = unstake_nfts(deps.as_mut(), &env, "sender", vec!["token1"]);
    assert_eq!(result, Err(InvalidStakingAsset));

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
    assert_eq!(result, Err(InvalidStakingAsset));

    // cannot unstake if they don't have it staked
    let result = unstake_nfts(deps.as_mut(), &env, "sender", vec!["token4"]);
    assert_eq!(
        result,
        Err(NoNftTokenStaked {
            token_id: "token4".to_string()
        })
    );

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

#[test]
fn unstake_nft_staked_by_another_user_fails() -> DaoResult<()> {
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
        None,
    )?;

    stake_nfts(&mut deps.as_mut(), &env, NFT_ADDR, "user1", vec!["token1"])?;

    // cannot unstake if another user staked it
    let result = unstake_nfts(deps.as_mut(), &env, "user2", vec!["token1"]);
    assert_eq!(result, Err(Unauthorized));

    Ok(())
}

#[test]
fn unstake_multisig_dao() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        multisig_dao_membership_info_with_members(&[("member", 100u64)]),
        None,
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
        None,
    )?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user", 100u8)?;

    create_stub_proposal(deps.as_mut(), &env, &info)?;

    vote_on_proposal(deps.as_mut(), &env, "user", 1, No)?;

    unstake_tokens(deps.as_mut(), &env, "user", 60u8)?;
    assert_proposal_result_amount(&mock_query_ctx(deps.as_ref(), &env), 1, No, 40u128);

    unstake_tokens(deps.as_mut(), &env, "user", 30u8)?;
    assert_proposal_result_amount(&mock_query_ctx(deps.as_ref(), &env), 1, No, 10u128);

    unstake_tokens(deps.as_mut(), &env, "user", 10u8)?;
    assert_proposal_result_amount(&mock_query_ctx(deps.as_ref(), &env), 1, No, 0u128);

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

    vote_on_proposal(deps.as_mut(), &env, "user", 1, Yes)?;

    unstake_nfts(deps.as_mut(), &env, "user", vec!["token2"])?;
    assert_proposal_result_amount(&mock_query_ctx(deps.as_ref(), &env), 1, Yes, 2u128);

    unstake_nfts(deps.as_mut(), &env, "user", vec!["token3"])?;
    assert_proposal_result_amount(&mock_query_ctx(deps.as_ref(), &env), 1, Yes, 1u128);

    unstake_nfts(deps.as_mut(), &env, "user", vec!["token1"])?;
    assert_proposal_result_amount(&mock_query_ctx(deps.as_ref(), &env), 1, Yes, 0u128);

    Ok(())
}

// TODO: unstaking doesn't reduce votes in ended proposals!
