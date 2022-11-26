use crate::contract::{execute, instantiate};
use crate::tests::helpers::{
    assert_member_voting_power, assert_total_stake, assert_user_nft_stake,
    assert_user_stake_is_none, assert_user_token_stake, existing_nft_dao_membership,
    existing_token_dao_membership, instantiate_stub_dao, stake_nfts, stake_tokens,
    stub_dao_gov_config, stub_dao_metadata, stub_enterprise_factory_contract, stub_token_info,
    CW20_ADDR, NFT_ADDR,
};
use crate::tests::querier::mock_querier::mock_dependencies;
use common::cw::testing::{mock_env, mock_info, mock_query_ctx};
use cosmwasm_std::{to_binary, Decimal};
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ReceiveMsg;
use enterprise_protocol::api::DaoMembershipInfo::Existing;
use enterprise_protocol::api::DaoType::Multisig;
use enterprise_protocol::api::ExistingDaoMembershipMsg;
use enterprise_protocol::error::DaoResult;
use enterprise_protocol::msg::{ExecuteMsg, InstantiateMsg};

#[test]
fn stake_token_dao() -> DaoResult<()> {
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

    assert_user_token_stake(mock_query_ctx(deps.as_ref(), &env), "sender", 0u8);
    assert_total_stake(mock_query_ctx(deps.as_ref(), &env), 0u8);

    let result = stake_nfts(
        &mut deps.as_mut(),
        &env,
        CW20_ADDR,
        "sender",
        vec!["token1"],
    );
    assert!(result.is_err());

    // random stake payload fails
    let result = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "sender".to_string(),
            amount: 1u8.into(),
            msg: to_binary(&1u8)?,
        }),
    );
    assert!(result.is_err());

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender", 12u8)?;
    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender", 31u8)?;
    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender1", 14u8)?;

    assert_user_token_stake(mock_query_ctx(deps.as_ref(), &env), "sender", 43u8);
    assert_user_token_stake(mock_query_ctx(deps.as_ref(), &env), "sender1", 14u8);
    assert_total_stake(mock_query_ctx(deps.as_ref(), &env), 57u8);

    Ok(())
}

#[test]
fn stake_nft_dao() -> DaoResult<()> {
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

    assert_user_nft_stake(mock_query_ctx(deps.as_ref(), &env), "sender", vec![]);
    assert_total_stake(mock_query_ctx(deps.as_ref(), &env), 0u8);

    let result = stake_tokens(deps.as_mut(), &env, NFT_ADDR, "sender", 1u8);
    assert!(result.is_err());

    // random stake payload fails
    let result = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            sender: "sender".to_string(),
            token_id: "token1".into(),
            msg: to_binary(&1u8)?,
        }),
    );
    assert!(result.is_err());

    stake_nfts(
        &mut deps.as_mut(),
        &env,
        NFT_ADDR,
        "sender",
        vec!["token1", "token2"],
    )?;
    stake_nfts(
        &mut deps.as_mut(),
        &env,
        NFT_ADDR,
        "sender1",
        vec!["token3"],
    )?;

    assert_user_nft_stake(
        mock_query_ctx(deps.as_ref(), &env),
        "sender",
        vec!["token1".to_string(), "token2".to_string()],
    );
    assert_user_nft_stake(
        mock_query_ctx(deps.as_ref(), &env),
        "sender1",
        vec!["token3".to_string()],
    );
    assert_total_stake(mock_query_ctx(deps.as_ref(), &env), 3u8);
    assert_member_voting_power(
        &mock_query_ctx(deps.as_ref(), &env),
        "sender",
        Decimal::from_ratio(2u8, 3u8),
    );
    assert_member_voting_power(
        &mock_query_ctx(deps.as_ref(), &env),
        "sender1",
        Decimal::from_ratio(1u8, 3u8),
    );

    Ok(())
}

// TODO: more than likely has to be deleted and replaced by an integration test, too difficult to mock
#[ignore]
#[test]
fn stake_multisig_dao() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let dao_gov_config = stub_dao_gov_config();
    let dao_cw3_membership_contract = "cw3_addr".to_string();
    deps.querier
        .with_multisig_members(&[("cw3_addr", &[("sender", 10u64)])]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            dao_metadata: stub_dao_metadata(),
            dao_gov_config: dao_gov_config.clone(),
            dao_membership_info: Existing(ExistingDaoMembershipMsg {
                dao_type: Multisig,
                membership_contract_addr: dao_cw3_membership_contract.clone(),
            }),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    )?;

    assert_user_stake_is_none(mock_query_ctx(deps.as_ref(), &env), "sender");
    assert_total_stake(mock_query_ctx(deps.as_ref(), &env), 0u8);

    let result = stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender", 1u8);
    assert!(result.is_err());

    let result = stake_nfts(
        &mut deps.as_mut(),
        &env,
        NFT_ADDR,
        "sender",
        vec!["token1", "token2"],
    );
    assert!(result.is_err());

    Ok(())
}
