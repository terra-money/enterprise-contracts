use crate::contract::{execute, query_claims, query_releasable_claims};
use crate::tests::helpers::{
    existing_nft_dao_membership, existing_token_dao_membership, instantiate_stub_dao, stake_nfts,
    stake_tokens, stub_dao_gov_config, stub_token_info, unstake_nfts, unstake_tokens, CW20_ADDR,
    NFT_ADDR,
};
use crate::tests::querier::mock_querier::mock_dependencies;
use common::cw::testing::{mock_env, mock_info, mock_query_ctx};
use cosmwasm_std::{wasm_execute, Addr, SubMsg, Timestamp};
use cw_asset::Asset;
use cw_utils::Duration;
use enterprise_protocol::api::{
    Claim, ClaimAsset, ClaimsParams, Cw20ClaimAsset, Cw721ClaimAsset, DaoGovConfig, ReleaseAt,
};
use enterprise_protocol::error::{DaoError, DaoResult};
use enterprise_protocol::msg::ExecuteMsg;
use DaoError::NothingToClaim;
use ReleaseAt::Height;

#[test]
fn unstaking_tokens_creates_claims() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.block.time = Timestamp::from_seconds(100);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        Some(DaoGovConfig {
            unlocking_period: Duration::Time(50),
            vote_duration: 10,
            ..stub_dao_gov_config()
        }),
    )?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender", 50u8)?;

    let claims = query_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(claims.claims.is_empty());

    unstake_tokens(deps.as_mut(), &env, "sender", 14u8)?;

    let claims = query_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert_eq!(
        claims.claims,
        vec![Claim {
            asset: ClaimAsset::Cw20(Cw20ClaimAsset {
                amount: 14u8.into()
            }),
            release_at: ReleaseAt::Timestamp(Timestamp::from_seconds(150))
        },]
    );
    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(releasable_claims.claims.is_empty());

    env.block.time = env.block.time.plus_seconds(20);

    unstake_tokens(deps.as_mut(), &env, "sender", 15u8)?;

    let claims = query_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert_eq!(
        claims.claims,
        vec![
            Claim {
                asset: ClaimAsset::Cw20(Cw20ClaimAsset {
                    amount: 14u8.into()
                }),
                release_at: ReleaseAt::Timestamp(Timestamp::from_seconds(150))
            },
            Claim {
                asset: ClaimAsset::Cw20(Cw20ClaimAsset {
                    amount: 15u8.into()
                }),
                release_at: ReleaseAt::Timestamp(Timestamp::from_seconds(170))
            },
        ]
    );
    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(releasable_claims.claims.is_empty());

    let claims = query_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "random_user".to_string(),
        },
    )?;
    assert!(claims.claims.is_empty());

    Ok(())
}

#[test]
fn unstaking_tokens_releases_claims_when_scheduled() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.block.height = 100u64;

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        Some(DaoGovConfig {
            unlocking_period: Duration::Height(50),
            ..stub_dao_gov_config()
        }),
    )?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender", 50u8)?;

    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(releasable_claims.claims.is_empty());

    unstake_tokens(deps.as_mut(), &env, "sender", 14u8)?;

    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(releasable_claims.claims.is_empty());

    env.block.height += 49;

    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(releasable_claims.claims.is_empty());

    unstake_tokens(deps.as_mut(), &env, "sender", 15u8)?;

    env.block.height += 1;

    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert_eq!(
        releasable_claims.claims,
        vec![Claim {
            asset: ClaimAsset::Cw20(Cw20ClaimAsset {
                amount: 14u8.into()
            }),
            release_at: Height(150u64.into())
        },]
    );

    env.block.height += 49;

    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert_eq!(
        releasable_claims.claims,
        vec![
            Claim {
                asset: ClaimAsset::Cw20(Cw20ClaimAsset {
                    amount: 14u8.into()
                }),
                release_at: Height(150u64.into())
            },
            Claim {
                asset: ClaimAsset::Cw20(Cw20ClaimAsset {
                    amount: 15u8.into()
                }),
                release_at: Height(199u64.into())
            },
        ]
    );

    Ok(())
}

#[test]
fn claiming_token_claims_sends_and_removes_them() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.block.time = Timestamp::from_seconds(100);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        Some(DaoGovConfig {
            unlocking_period: Duration::Time(50),
            vote_duration: 50,
            ..stub_dao_gov_config()
        }),
    )?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender", 50u8)?;

    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(releasable_claims.claims.is_empty());

    unstake_tokens(deps.as_mut(), &env, "sender", 14u8)?;
    unstake_tokens(deps.as_mut(), &env, "sender", 15u8)?;

    env.block.time = env.block.time.plus_seconds(50);

    // users with no releasable claims cannot claim
    let result = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("random_user", &vec![]),
        ExecuteMsg::Claim {},
    );
    assert_eq!(result, Err(NothingToClaim));

    let response = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("sender", &vec![]),
        ExecuteMsg::Claim {},
    )?;

    assert_eq!(
        response.messages,
        vec![
            SubMsg::new(Asset::cw20(Addr::unchecked(CW20_ADDR), 14u8).transfer_msg("sender")?),
            SubMsg::new(Asset::cw20(Addr::unchecked(CW20_ADDR), 15u8).transfer_msg("sender")?),
        ]
    );

    let claims = query_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(claims.claims.is_empty());

    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(releasable_claims.claims.is_empty());

    Ok(())
}

#[test]
fn unstaking_nfts_creates_claims() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.block.time = Timestamp::from_seconds(100);

    deps.querier.with_num_tokens(&[(NFT_ADDR, 100u64)]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_nft_dao_membership(NFT_ADDR),
        Some(DaoGovConfig {
            unlocking_period: Duration::Time(50),
            vote_duration: 10,
            ..stub_dao_gov_config()
        }),
    )?;

    stake_nfts(
        &mut deps.as_mut(),
        &env,
        NFT_ADDR,
        "sender",
        vec!["token1", "token2"],
    )?;

    let claims = query_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(claims.claims.is_empty());

    unstake_nfts(deps.as_mut(), &env, "sender", vec!["token2"])?;

    let claims = query_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert_eq!(
        claims.claims,
        vec![Claim {
            asset: ClaimAsset::Cw721(Cw721ClaimAsset {
                tokens: vec!["token2".to_string()],
            }),
            release_at: ReleaseAt::Timestamp(Timestamp::from_seconds(150))
        }]
    );
    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(releasable_claims.claims.is_empty());

    env.block.time = env.block.time.plus_seconds(20);

    unstake_nfts(deps.as_mut(), &env, "sender", vec!["token1"])?;

    let claims = query_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert_eq!(
        claims.claims,
        vec![
            Claim {
                asset: ClaimAsset::Cw721(Cw721ClaimAsset {
                    tokens: vec!["token2".to_string()],
                }),
                release_at: ReleaseAt::Timestamp(Timestamp::from_seconds(150))
            },
            Claim {
                asset: ClaimAsset::Cw721(Cw721ClaimAsset {
                    tokens: vec!["token1".to_string()],
                }),
                release_at: ReleaseAt::Timestamp(Timestamp::from_seconds(170))
            },
        ]
    );
    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(releasable_claims.claims.is_empty());

    let claims = query_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "random_user".to_string(),
        },
    )?;
    assert!(claims.claims.is_empty());

    Ok(())
}

#[test]
fn unstaking_nfts_releases_claims_when_scheduled() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.block.time = Timestamp::from_seconds(100u64);

    deps.querier.with_num_tokens(&[(NFT_ADDR, 100u64)]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_nft_dao_membership(NFT_ADDR),
        Some(DaoGovConfig {
            unlocking_period: Duration::Time(50),
            vote_duration: 40,
            ..stub_dao_gov_config()
        }),
    )?;

    stake_nfts(
        &mut deps.as_mut(),
        &env,
        NFT_ADDR,
        "sender",
        vec!["token1", "token2"],
    )?;

    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(releasable_claims.claims.is_empty());

    unstake_nfts(deps.as_mut(), &env, "sender", vec!["token1"])?;

    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(releasable_claims.claims.is_empty());

    env.block.time = env.block.time.plus_seconds(49);

    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(releasable_claims.claims.is_empty());

    unstake_nfts(deps.as_mut(), &env, "sender", vec!["token2"])?;

    env.block.time = env.block.time.plus_seconds(1);

    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert_eq!(
        releasable_claims.claims,
        vec![Claim {
            asset: ClaimAsset::Cw721(Cw721ClaimAsset {
                tokens: vec!["token1".to_string()],
            }),
            release_at: ReleaseAt::Timestamp(Timestamp::from_seconds(150u64))
        },]
    );

    env.block.time = env.block.time.plus_seconds(49);

    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert_eq!(
        releasable_claims.claims,
        vec![
            Claim {
                asset: ClaimAsset::Cw721(Cw721ClaimAsset {
                    tokens: vec!["token1".to_string()],
                }),
                release_at: ReleaseAt::Timestamp(Timestamp::from_seconds(150u64))
            },
            Claim {
                asset: ClaimAsset::Cw721(Cw721ClaimAsset {
                    tokens: vec!["token2".to_string()],
                }),
                release_at: ReleaseAt::Timestamp(Timestamp::from_seconds(199u64))
            },
        ]
    );

    Ok(())
}

#[test]
fn claiming_nft_claims_sends_and_removes_them() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.block.time = Timestamp::from_seconds(100);

    deps.querier.with_num_tokens(&[(NFT_ADDR, 100u64)]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_nft_dao_membership(NFT_ADDR),
        Some(DaoGovConfig {
            unlocking_period: Duration::Time(50),
            vote_duration: 50,
            ..stub_dao_gov_config()
        }),
    )?;

    stake_nfts(
        &mut deps.as_mut(),
        &env,
        NFT_ADDR,
        "sender",
        vec!["token1", "token2"],
    )?;

    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(releasable_claims.claims.is_empty());

    unstake_nfts(deps.as_mut(), &env, "sender", vec!["token1"])?;
    unstake_nfts(deps.as_mut(), &env, "sender", vec!["token2"])?;

    env.block.time = env.block.time.plus_seconds(50);

    // users with no releasable claims cannot claim
    let result = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("random_user", &vec![]),
        ExecuteMsg::Claim {},
    );
    assert_eq!(result, Err(NothingToClaim));

    let response = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("sender", &vec![]),
        ExecuteMsg::Claim {},
    )?;

    assert_eq!(
        response.messages,
        vec![
            SubMsg::new(wasm_execute(
                NFT_ADDR,
                &cw721::Cw721ExecuteMsg::TransferNft {
                    recipient: "sender".to_string(),
                    token_id: "token1".to_string()
                },
                vec![]
            )?),
            SubMsg::new(wasm_execute(
                NFT_ADDR,
                &cw721::Cw721ExecuteMsg::TransferNft {
                    recipient: "sender".to_string(),
                    token_id: "token2".to_string()
                },
                vec![]
            )?),
        ]
    );

    let claims = query_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(claims.claims.is_empty());

    let releasable_claims = query_releasable_claims(
        mock_query_ctx(deps.as_ref(), &env),
        ClaimsParams {
            owner: "sender".to_string(),
        },
    )?;
    assert!(releasable_claims.claims.is_empty());

    Ok(())
}
