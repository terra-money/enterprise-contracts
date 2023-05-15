use crate::tests::helpers::{
    assert_total_stake, assert_user_nft_stake, assert_user_token_stake,
    existing_nft_dao_membership, existing_token_dao_membership, instantiate_stub_dao,
    multisig_dao_membership_info_with_members, stake_nfts, stake_tokens, stub_token_info,
    unstake_nfts, unstake_tokens, CW20_ADDR, NFT_ADDR,
};
use crate::tests::querier::mock_querier::mock_dependencies;
use common::cw::testing::{mock_env, mock_info, mock_query_ctx};
use enterprise_protocol::error::DaoError::{InvalidStakingAsset, Unauthorized};
use enterprise_protocol::error::DaoResult;

#[test]
fn unstake_token_dao() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[("cw20_addr", &stub_token_info())]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
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
fn unstake_nft_staked_by_another_user_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier.with_num_tokens(&[(NFT_ADDR, 100u64)]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
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
        &mut deps.as_mut(),
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
