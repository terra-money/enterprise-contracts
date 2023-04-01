use crate::contract::{
    execute, instantiate, query_asset_whitelist, query_dao_info, query_list_multisig_members,
    query_nft_whitelist, query_proposal,
};
use crate::tests::helpers::{
    assert_member_voting_power, assert_proposal_result_amount, assert_proposal_status,
    create_proposal, create_stub_proposal, existing_nft_dao_membership,
    existing_token_dao_membership, instantiate_stub_dao, multisig_dao_membership_info_with_members,
    stake_nfts, stake_tokens, stub_dao_gov_config, stub_dao_metadata,
    stub_enterprise_factory_contract, stub_token_info, unstake_nfts, unstake_tokens,
    vote_on_proposal, CW20_ADDR, DAO_ADDR, ENTERPRISE_GOVERNANCE_CODE_ID,
    FUNDS_DISTRIBUTOR_CODE_ID, NFT_ADDR, PROPOSAL_DESCRIPTION, PROPOSAL_TITLE,
};
use crate::tests::querier::mock_querier::mock_dependencies;
use common::cw::testing::{mock_env, mock_info, mock_query_ctx};
use cosmwasm_std::{
    coins, to_binary, Addr, Attribute, BankMsg, Decimal, SubMsg, Timestamp, Uint128, WasmMsg,
};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfo};
use cw_utils::Duration;
use enterprise_protocol::api::ModifyValue::Change;
use enterprise_protocol::api::ProposalAction::{
    ExecuteMsgs, ModifyMultisigMembership, RequestFundingFromDao, UpdateCouncil, UpdateMetadata,
    UpdateNftWhitelist, UpgradeDao,
};
use enterprise_protocol::api::ProposalStatus::{Passed, Rejected};
use enterprise_protocol::api::{
    CreateProposalMsg, DaoCouncil, DaoCouncilSpec, DaoGovConfig, DaoMetadata, DaoSocialData,
    ExecuteMsgsMsg, ExecuteProposalMsg, ListMultisigMembersMsg, Logo, ModifyMultisigMembershipMsg,
    MultisigMember, ProposalAction, ProposalActionType, ProposalParams, RequestFundingFromDaoMsg,
    UpdateAssetWhitelistMsg, UpdateCouncilMsg, UpdateGovConfigMsg, UpdateMetadataMsg,
    UpdateNftWhitelistMsg, UpgradeDaoMsg,
};
use enterprise_protocol::error::DaoResult;
use enterprise_protocol::msg::ExecuteMsg::{ExecuteProposal, Receive};
use enterprise_protocol::msg::{Cw20HookMsg, InstantiateMsg, MigrateMsg};
use poll_engine_api::api::VoteOutcome::{Abstain, No, Veto, Yes};
use ProposalAction::{UpdateAssetWhitelist, UpdateGovConfig};

// TODO: think of an elegant way to mock out Enterprise gov contract

#[ignore]
#[test]
fn execute_proposal_with_outcome_no_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
        None,
    )?;

    create_stub_proposal(deps.as_mut(), &env, &info)?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender", 100u128)?;

    vote_on_proposal(deps.as_mut(), &env, "sender", 1, No)?;

    let result = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 1 }),
    );

    assert!(result.is_err());

    Ok(())
}

#[ignore]
#[test]
fn execute_proposal_with_outcome_yes_but_not_ended_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.block.time = Timestamp::from_seconds(12000);
    let dao_gov_config = DaoGovConfig {
        vote_duration: 1000,
        ..stub_dao_gov_config()
    };

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        Some(dao_gov_config.clone()),
        None,
    )?;

    create_stub_proposal(deps.as_mut(), &env, &info)?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender", 100u128)?;

    vote_on_proposal(deps.as_mut(), &env, "sender", 1, Yes)?;

    env.block.time = env.block.time.plus_seconds(999);

    let result = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 1 }),
    );

    assert!(result.is_err());

    Ok(())
}

#[ignore]
#[test]
fn execute_proposal_with_outcome_yes_and_ended_executes_proposal_actions() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.contract.address = Addr::unchecked(DAO_ADDR);
    env.block.time = Timestamp::from_seconds(12000);
    let dao_gov_config = DaoGovConfig {
        vote_duration: 1000,
        quorum: Decimal::from_ratio(1u8, 10u8),
        threshold: Decimal::from_ratio(2u8, 10u8),
        unlocking_period: Duration::Time(1000),
        minimum_deposit: None,
        veto_threshold: None,
        allow_early_proposal_execution: false,
    };

    let enterprise_factory_contract = stub_enterprise_factory_contract();
    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);
    deps.querier
        .with_enterprise_code_ids(&[(&enterprise_factory_contract, &[7u64])]);

    let token1 = AssetInfo::cw20(Addr::unchecked("token1"));
    let token2 = AssetInfo::cw20(Addr::unchecked("token2"));
    let token3 = AssetInfo::cw20(Addr::unchecked("token3"));

    let nft1 = Addr::unchecked("nft1");
    let nft2 = Addr::unchecked("nft2");
    let nft3 = Addr::unchecked("nft3");

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
            funds_distributor_code_id: FUNDS_DISTRIBUTOR_CODE_ID,
            dao_metadata: stub_dao_metadata(),
            dao_gov_config: dao_gov_config.clone(),
            dao_council: None,
            dao_membership_info: existing_token_dao_membership(CW20_ADDR),
            enterprise_factory_contract,
            asset_whitelist: Some(vec![token1.clone(), token2.clone()]),
            nft_whitelist: Some(vec![nft1.clone(), nft2.clone()]),
            minimum_weight_for_rewards: None,
        },
    )?;

    let migrate_msg = to_binary(&MigrateMsg {})?;

    let new_dao_council = Some(DaoCouncilSpec {
        members: vec!["new_member1".to_string(), "new_member2".to_string()],
        quorum: Decimal::percent(75),
        threshold: Decimal::percent(50),
        allowed_proposal_action_types: Some(vec![ProposalActionType::UpdateMetadata]),
    });

    let proposal_actions = vec![
        UpdateGovConfig(UpdateGovConfigMsg {
            quorum: Change(Decimal::percent(30)),
            threshold: Change(Decimal::percent(40)),
            veto_threshold: Change(Some(Decimal::percent(37))),
            voting_duration: Change(10u64.into()),
            unlocking_period: Change(Duration::Height(10)),
            minimum_deposit: Change(Some(Uint128::one())),
            allow_early_proposal_execution: Change(true),
        }),
        UpdateAssetWhitelist(UpdateAssetWhitelistMsg {
            // TODO: use this
            // add: vec![token1.clone(), token3.clone()],
            add: vec![token3.clone()],
            remove: vec![token2.clone()],
        }),
        UpdateNftWhitelist(UpdateNftWhitelistMsg {
            add: vec![nft2.clone(), nft3.clone()],
            remove: vec![nft1.clone()],
        }),
        UpdateMetadata(UpdateMetadataMsg {
            name: Change("Updated name".to_string()),
            description: Change(Some("Updated description".to_string())),
            logo: Change(Logo::Url("updated_logo_url".to_string())),
            github_username: Change(Some("updated_github".to_string())),
            discord_username: Change(Some("updated_discord".to_string())),
            twitter_username: Change(Some("updated_twitter".to_string())),
            telegram_username: Change(Some("updated_telegram".to_string())),
        }),
        RequestFundingFromDao(RequestFundingFromDaoMsg {
            recipient: "recipient".to_string(),
            assets: vec![
                Asset::cw20(Addr::unchecked("token1"), 200u128),
                Asset::native(Addr::unchecked("uluna"), 300u128),
            ],
        }),
        UpgradeDao(UpgradeDaoMsg {
            new_dao_code_id: 7,
            migrate_msg: migrate_msg.clone(),
        }),
        ExecuteMsgs(ExecuteMsgsMsg {
            action_type: "execute_and_send".to_string(),
            msgs: vec![
                "{\"wasm\": { \"execute\": { \"contract_addr\": \"execute_addr\", \"msg\": \"InsgXCJ0ZXN0X21zZ1wiOiB7IFwiaWRcIjogXCIxMjNcIiB9IH0i\", \"funds\": [] } } }".to_string(),
                "{\"bank\": { \"send\": { \"to_address\": \"send_addr\", \"amount\": [{\"amount\": \"123456789\", \"denom\": \"some_denom\"}]} } }".to_string()
            ],
        }),
        UpdateCouncil(UpdateCouncilMsg { dao_council: new_dao_council.clone() }),
    ];

    let response = create_proposal(
        deps.as_mut(),
        &env,
        &info,
        None,
        None,
        proposal_actions.clone(),
    )?;

    assert_eq!(
        response.attributes,
        vec![
            Attribute::new("action", "create_proposal"),
            Attribute::new("dao_address", DAO_ADDR),
            Attribute::new("proposal_id", "1"),
        ]
    );

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender", 100u128)?;

    vote_on_proposal(deps.as_mut(), &env, "sender", 1, Yes)?;

    env.block.time = env.block.time.plus_seconds(1000);

    let response = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 1 }),
    )?;

    assert_eq!(
        response.messages,
        vec![
            SubMsg::new(Asset::cw20(Addr::unchecked("token1"), 200u128).transfer_msg("recipient")?),
            SubMsg::new(
                Asset::native(Addr::unchecked("uluna"), 300u128).transfer_msg("recipient")?
            ),
            SubMsg::new(WasmMsg::Migrate {
                contract_addr: DAO_ADDR.to_string(),
                new_code_id: 7,
                msg: migrate_msg,
            }),
            SubMsg::new(WasmMsg::Execute {
                contract_addr: "execute_addr".to_string(),
                msg: to_binary("{ \"test_msg\": { \"id\": \"123\" } }")?,
                funds: vec![],
            }),
            SubMsg::new(BankMsg::Send {
                to_address: "send_addr".to_string(),
                amount: coins(123456789, "some_denom")
            }),
        ]
    );

    let dao_info = query_dao_info(mock_query_ctx(deps.as_ref(), &env))?;
    assert_eq!(
        dao_info.metadata,
        DaoMetadata {
            name: "Updated name".to_string(),
            description: Some("Updated description".to_string()),
            logo: Logo::Url("updated_logo_url".to_string()),
            socials: DaoSocialData {
                github_username: Some("updated_github".to_string()),
                discord_username: Some("updated_discord".to_string()),
                twitter_username: Some("updated_twitter".to_string()),
                telegram_username: Some("updated_telegram".to_string()),
            },
        }
    );
    assert_eq!(
        dao_info.dao_council,
        Some(DaoCouncil {
            members: vec![
                Addr::unchecked("new_member1"),
                Addr::unchecked("new_member2")
            ],
            allowed_proposal_action_types: vec![ProposalActionType::UpdateMetadata],
            quorum: Decimal::percent(75),
            threshold: Decimal::percent(50),
        })
    );
    assert_eq!(
        dao_info.gov_config,
        DaoGovConfig {
            quorum: Decimal::percent(30),
            threshold: Decimal::percent(40),
            veto_threshold: Some(Decimal::percent(37)),
            vote_duration: 10u64.into(),
            unlocking_period: Duration::Height(10),
            minimum_deposit: Some(Uint128::one()),
            allow_early_proposal_execution: true,
        }
    );

    let asset_whitelist = query_asset_whitelist(mock_query_ctx(deps.as_ref(), &env))?;
    assert_eq!(asset_whitelist.assets, vec![token1, token3]);

    let nft_whitelist = query_nft_whitelist(mock_query_ctx(deps.as_ref(), &env))?;
    assert_eq!(nft_whitelist.nfts, vec![nft2, nft3]);

    // ensure proposal actions were not removed after execution
    let proposal = query_proposal(
        mock_query_ctx(deps.as_ref(), &env),
        ProposalParams { proposal_id: 1 },
    )?;
    assert_eq!(proposal.proposal.proposal_actions, proposal_actions);

    Ok(())
}

#[ignore]
#[test]
fn execute_passed_proposal_to_update_multisig_members_changes_membership() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.block.time = Timestamp::from_seconds(12000);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        multisig_dao_membership_info_with_members(&[
            ("member1", 1u64),
            ("member2", 2u64),
            ("member3", 3u64),
        ]),
        None,
        None,
    )?;

    let proposal_actions = vec![ModifyMultisigMembership(ModifyMultisigMembershipMsg {
        edit_members: vec![
            MultisigMember {
                address: "member1".to_string(),
                weight: 0u128.into(),
            },
            MultisigMember {
                address: "member3".to_string(),
                weight: 5u128.into(),
            },
            MultisigMember {
                address: "member4".to_string(),
                weight: 4u128.into(),
            },
        ],
    })];

    create_proposal(
        deps.as_mut(),
        &env,
        &mock_info("member1", &vec![]),
        Some(PROPOSAL_TITLE),
        Some(PROPOSAL_DESCRIPTION),
        proposal_actions.clone(),
    )?;

    vote_on_proposal(deps.as_mut(), &env, "member3", 1, Yes)?;

    env.block.time = env.block.time.plus_seconds(1000);

    execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 1 }),
    )?;

    let qctx = mock_query_ctx(deps.as_ref(), &env);
    assert_member_voting_power(&qctx, "member1", Decimal::zero());
    assert_member_voting_power(&qctx, "member2", Decimal::from_ratio(2u8, 11u8));
    assert_member_voting_power(&qctx, "member3", Decimal::from_ratio(5u8, 11u8));
    assert_member_voting_power(&qctx, "member4", Decimal::from_ratio(4u8, 11u8));

    let list_members = query_list_multisig_members(
        mock_query_ctx(deps.as_ref(), &env),
        ListMultisigMembersMsg {
            start_after: None,
            limit: None,
        },
    )?;
    assert_eq!(
        list_members.members,
        vec![
            MultisigMember {
                address: "member2".to_string(),
                weight: 2u8.into()
            },
            MultisigMember {
                address: "member3".to_string(),
                weight: 5u8.into()
            },
            MultisigMember {
                address: "member4".to_string(),
                weight: 4u8.into()
            },
        ]
    );

    Ok(())
}

#[ignore]
#[test]
fn execute_passed_proposal_to_update_multisig_members_does_not_change_votes_on_ended_proposals(
) -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.block.time = Timestamp::from_seconds(12000);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        multisig_dao_membership_info_with_members(&[
            ("member1", 1u64),
            ("member2", 2u64),
            ("member3", 3u64),
        ]),
        Some(DaoGovConfig {
            vote_duration: 1000,
            ..stub_dao_gov_config()
        }),
        None,
    )?;

    create_stub_proposal(deps.as_mut(), &env, &mock_info("member1", &vec![]))?;

    vote_on_proposal(deps.as_mut(), &env, "member3", 1, Yes)?;

    create_proposal(
        deps.as_mut(),
        &env,
        &mock_info("member1", &vec![]),
        None,
        None,
        vec![ModifyMultisigMembership(ModifyMultisigMembershipMsg {
            edit_members: vec![
                MultisigMember {
                    address: "member1".to_string(),
                    weight: 0u128.into(),
                },
                MultisigMember {
                    address: "member3".to_string(),
                    weight: 5u128.into(),
                },
                MultisigMember {
                    address: "member4".to_string(),
                    weight: 4u128.into(),
                },
            ],
        })],
    )?;

    vote_on_proposal(deps.as_mut(), &env, "member3", 2, Yes)?;

    env.block.time = env.block.time.plus_seconds(1000);

    execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 2 }),
    )?;

    let qctx = mock_query_ctx(deps.as_ref(), &env);
    assert_proposal_result_amount(&qctx, 1, Yes, 3u128);

    let proposal = query_proposal(qctx, ProposalParams { proposal_id: 1 })?;
    assert_eq!(proposal.total_votes_available, Uint128::from(6u8));

    Ok(())
}

#[ignore]
#[test]
fn execute_passed_proposal_to_update_multisig_members_updates_votes_on_active_proposals(
) -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.block.time = Timestamp::from_seconds(12000);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        multisig_dao_membership_info_with_members(&[
            ("member1", 1u64),
            ("member2", 2u64),
            ("member3", 3u64),
        ]),
        Some(DaoGovConfig {
            vote_duration: 1000,
            ..stub_dao_gov_config()
        }),
        None,
    )?;

    create_proposal(
        deps.as_mut(),
        &env,
        &mock_info("member1", &vec![]),
        None,
        None,
        vec![ModifyMultisigMembership(ModifyMultisigMembershipMsg {
            edit_members: vec![
                MultisigMember {
                    address: "member1".to_string(),
                    weight: 0u128.into(),
                },
                MultisigMember {
                    address: "member3".to_string(),
                    weight: 5u128.into(),
                },
                MultisigMember {
                    address: "member4".to_string(),
                    weight: 4u128.into(),
                },
            ],
        })],
    )?;

    vote_on_proposal(deps.as_mut(), &env, "member3", 1, Yes)?;

    env.block.time = env.block.time.plus_seconds(10);

    create_stub_proposal(deps.as_mut(), &env, &mock_info("member1", &vec![]))?;

    vote_on_proposal(deps.as_mut(), &env, "member1", 2, No)?;
    vote_on_proposal(deps.as_mut(), &env, "member2", 2, No)?;
    vote_on_proposal(deps.as_mut(), &env, "member3", 2, Yes)?;

    env.block.time = env.block.time.plus_seconds(990);

    execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 1 }),
    )?;

    let qctx = mock_query_ctx(deps.as_ref(), &env);
    assert_proposal_result_amount(&qctx, 2, Yes, 5u128);
    assert_proposal_result_amount(&qctx, 2, No, 2u128);

    let proposal = query_proposal(qctx, ProposalParams { proposal_id: 2 })?;
    assert_eq!(proposal.total_votes_available, Uint128::from(11u8));

    Ok(())
}

#[ignore]
#[test]
fn execute_proposal_with_outcome_yes_refunds_token_deposits() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.block.time = Timestamp::from_seconds(12000);
    let dao_gov_config = DaoGovConfig {
        vote_duration: 1000,
        quorum: Decimal::from_ratio(1u8, 10u8),
        threshold: Decimal::from_ratio(2u8, 10u8),
        unlocking_period: Duration::Time(1000),
        minimum_deposit: None,
        veto_threshold: None,
        allow_early_proposal_execution: false,
    };

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        Some(dao_gov_config.clone()),
        None,
    )?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user", 300u128)?;

    let create_proposal_msg = CreateProposalMsg {
        title: "Proposal title".to_string(),
        description: Some("Description".to_string()),
        proposal_actions: vec![],
    };
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info(CW20_ADDR, &vec![]),
        Receive(Cw20ReceiveMsg {
            sender: "user".to_string(),
            amount: 50u128.into(),
            msg: to_binary(&Cw20HookMsg::CreateProposal(create_proposal_msg))?,
        }),
    )?;

    vote_on_proposal(deps.as_mut(), &env, "user", 1, Yes)?;

    env.block.time = env.block.time.plus_seconds(1000);

    let response = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 1 }),
    )?;

    assert_eq!(
        response.messages,
        vec![SubMsg::new(
            Asset::cw20(Addr::unchecked(CW20_ADDR), 50u128).transfer_msg("user")?
        )]
    );

    Ok(())
}

#[ignore]
#[test]
fn execute_proposal_with_outcome_no_refunds_token_deposits() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.block.time = Timestamp::from_seconds(12000);
    let dao_gov_config = DaoGovConfig {
        vote_duration: 1000,
        quorum: Decimal::from_ratio(1u8, 10u8),
        threshold: Decimal::from_ratio(2u8, 10u8),
        unlocking_period: Duration::Time(1000),
        minimum_deposit: None,
        veto_threshold: None,
        allow_early_proposal_execution: false,
    };

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        Some(dao_gov_config.clone()),
        None,
    )?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user", 300u128)?;

    let create_proposal_msg = CreateProposalMsg {
        title: "Proposal title".to_string(),
        description: Some("Description".to_string()),
        proposal_actions: vec![],
    };
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info(CW20_ADDR, &vec![]),
        Receive(Cw20ReceiveMsg {
            sender: "user".to_string(),
            amount: 50u128.into(),
            msg: to_binary(&Cw20HookMsg::CreateProposal(create_proposal_msg))?,
        }),
    )?;

    vote_on_proposal(deps.as_mut(), &env, "user", 1, No)?;

    env.block.time = env.block.time.plus_seconds(1000);

    let response = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 1 }),
    )?;

    assert_eq!(
        response.messages,
        vec![SubMsg::new(
            Asset::cw20(Addr::unchecked(CW20_ADDR), 50u128).transfer_msg("user")?
        )]
    );

    Ok(())
}

#[ignore]
#[test]
fn execute_proposal_with_threshold_not_reached_refunds_token_deposits() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.block.time = Timestamp::from_seconds(12000);
    let dao_gov_config = DaoGovConfig {
        vote_duration: 1000,
        quorum: Decimal::percent(10),
        threshold: Decimal::percent(20),
        unlocking_period: Duration::Time(1000),
        minimum_deposit: None,
        veto_threshold: None,
        allow_early_proposal_execution: false,
    };

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        Some(dao_gov_config.clone()),
        None,
    )?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user", 300u128)?;

    let create_proposal_msg = CreateProposalMsg {
        title: "Proposal title".to_string(),
        description: Some("Description".to_string()),
        proposal_actions: vec![],
    };
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info(CW20_ADDR, &vec![]),
        Receive(Cw20ReceiveMsg {
            sender: "user".to_string(),
            amount: 50u128.into(),
            msg: to_binary(&Cw20HookMsg::CreateProposal(create_proposal_msg))?,
        }),
    )?;

    vote_on_proposal(deps.as_mut(), &env, "user", 1, Abstain)?;

    env.block.time = env.block.time.plus_seconds(1000);

    let response = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 1 }),
    )?;

    assert_eq!(
        response.messages,
        vec![SubMsg::new(
            Asset::cw20(Addr::unchecked(CW20_ADDR), 50u128).transfer_msg("user")?
        )]
    );

    Ok(())
}

#[ignore]
#[test]
fn execute_proposal_with_quorum_not_reached_does_not_refund_token_deposits() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.block.time = Timestamp::from_seconds(12000);
    let dao_gov_config = DaoGovConfig {
        vote_duration: 1000,
        quorum: Decimal::percent(10),
        threshold: Decimal::percent(20),
        unlocking_period: Duration::Time(1000),
        minimum_deposit: None,
        veto_threshold: None,
        allow_early_proposal_execution: false,
    };

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        Some(dao_gov_config.clone()),
        None,
    )?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user", 300u128)?;

    let create_proposal_msg = CreateProposalMsg {
        title: "Proposal title".to_string(),
        description: Some("Description".to_string()),
        proposal_actions: vec![],
    };
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info(CW20_ADDR, &vec![]),
        Receive(Cw20ReceiveMsg {
            sender: "user".to_string(),
            amount: 50u128.into(),
            msg: to_binary(&Cw20HookMsg::CreateProposal(create_proposal_msg))?,
        }),
    )?;

    env.block.time = env.block.time.plus_seconds(1000);

    let response = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 1 }),
    )?;

    assert!(response.messages.is_empty());

    Ok(())
}

#[ignore]
#[test]
fn execute_proposal_with_outcome_veto_does_not_refund_token_deposits() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.block.time = Timestamp::from_seconds(12000);
    let dao_gov_config = DaoGovConfig {
        vote_duration: 1000,
        quorum: Decimal::from_ratio(1u8, 10u8),
        threshold: Decimal::from_ratio(2u8, 10u8),
        unlocking_period: Duration::Time(1000),
        minimum_deposit: None,
        veto_threshold: None,
        allow_early_proposal_execution: false,
    };

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        Some(dao_gov_config.clone()),
        None,
    )?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user", 300u128)?;

    let create_proposal_msg = CreateProposalMsg {
        title: "Proposal title".to_string(),
        description: Some("Description".to_string()),
        proposal_actions: vec![],
    };
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info(CW20_ADDR, &vec![]),
        Receive(Cw20ReceiveMsg {
            sender: "user".to_string(),
            amount: 50u128.into(),
            msg: to_binary(&Cw20HookMsg::CreateProposal(create_proposal_msg))?,
        }),
    )?;

    vote_on_proposal(deps.as_mut(), &env, "user", 1, Veto)?;

    env.block.time = env.block.time.plus_seconds(1000);

    let response = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 1 }),
    )?;

    assert!(response.messages.is_empty());

    Ok(())
}

#[ignore]
#[test]
fn execute_proposal_that_was_executed_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    env.block.time = Timestamp::from_seconds(12000);
    let dao_gov_config = DaoGovConfig {
        vote_duration: 1000,
        ..stub_dao_gov_config()
    };

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        Some(dao_gov_config.clone()),
        None,
    )?;

    create_stub_proposal(deps.as_mut(), &env, &info)?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "sender", 100u128)?;

    vote_on_proposal(deps.as_mut(), &env, "sender", 1, Yes)?;

    env.block.time = env.block.time.plus_seconds(1000);

    execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 1 }),
    )?;

    let result = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 1 }),
    );

    assert!(result.is_err());

    Ok(())
}

#[ignore]
#[test]
fn proposal_stores_total_votes_available_at_expiration_if_not_executed_before() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("user", &[]);

    env.block.time = Timestamp::from_seconds(12000);
    let dao_gov_config = DaoGovConfig {
        vote_duration: 1000,
        ..stub_dao_gov_config()
    };

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        Some(dao_gov_config.clone()),
        None,
    )?;

    create_stub_proposal(deps.as_mut(), &env, &info)?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user1", 100u128)?;
    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user2", 70u128)?;

    vote_on_proposal(deps.as_mut(), &env, "user1", 1, Yes)?;

    env.block.time = env.block.time.plus_seconds(400);

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user2", 20u128)?;

    let proposal = query_proposal(
        mock_query_ctx(deps.as_ref(), &env),
        ProposalParams { proposal_id: 1 },
    )?;
    assert_eq!(proposal.total_votes_available, Uint128::from(190u128));

    env.block.time = env.block.time.plus_seconds(200);

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user2", 5u128)?;

    env.block.time = env.block.time.plus_seconds(401);

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user2", 200u128)?;

    let proposal = query_proposal(
        mock_query_ctx(deps.as_ref(), &env),
        ProposalParams { proposal_id: 1 },
    )?;
    assert_eq!(proposal.total_votes_available, Uint128::from(195u128));

    let result = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 1 }),
    );

    assert!(result.is_ok());

    env.block.time = env.block.time.plus_seconds(100);

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user2", 200u128)?;

    let proposal = query_proposal(
        mock_query_ctx(deps.as_ref(), &env),
        ProposalParams { proposal_id: 1 },
    )?;
    assert_eq!(proposal.total_votes_available, Uint128::from(195u128));

    Ok(())
}

#[ignore]
#[test]
fn execute_proposal_uses_total_votes_available_at_expiration() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("user", &[]);

    env.block.time = Timestamp::from_seconds(12000);
    let dao_gov_config = DaoGovConfig {
        vote_duration: 1000,
        quorum: Decimal::from_ratio(50u8, 100u8),
        ..stub_dao_gov_config()
    };

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        Some(dao_gov_config.clone()),
        None,
    )?;

    create_stub_proposal(deps.as_mut(), &env, &info)?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user1", 100u128)?;
    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user2", 200u128)?;

    vote_on_proposal(deps.as_mut(), &env, "user1", 1, Yes)?;

    env.block.time = env.block.time.plus_seconds(1000);

    unstake_tokens(deps.as_mut(), &env, "user2", 200u128)?;
    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user3", 400u128)?;

    let proposal = query_proposal(
        mock_query_ctx(deps.as_ref(), &env),
        ProposalParams { proposal_id: 1 },
    )?;
    assert_eq!(proposal.total_votes_available, Uint128::from(300u128));

    execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 1 }),
    )?;

    // has to be rejected, since at the time of expiration, there were 300 total votes, 100 cast, and quorum is 50%
    assert_proposal_status(&mock_query_ctx(deps.as_ref(), &env), 1, Rejected);

    Ok(())
}

#[ignore]
#[test]
fn execute_proposal_uses_total_votes_available_at_expiration_nft() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("user", &[]);

    env.block.time = Timestamp::from_seconds(12000);
    let dao_gov_config = DaoGovConfig {
        vote_duration: 1000,
        quorum: Decimal::from_ratio(50u8, 100u8),
        threshold: Decimal::from_ratio(50u8, 100u8),
        ..stub_dao_gov_config()
    };

    deps.querier.with_num_tokens(&[(NFT_ADDR, 100u64)]);

    deps.querier
        .with_nft_holders(&[(NFT_ADDR, &[("user1", &["token1"])])]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        existing_nft_dao_membership(NFT_ADDR),
        Some(dao_gov_config),
        None,
    )?;

    create_stub_proposal(deps.as_mut(), &env, &mock_info("user1", &vec![]))?;

    stake_nfts(&mut deps.as_mut(), &env, NFT_ADDR, "user1", vec!["token1"])?;
    stake_nfts(
        &mut deps.as_mut(),
        &env,
        NFT_ADDR,
        "user2",
        vec!["token2", "token3"],
    )?;

    vote_on_proposal(deps.as_mut(), &env, "user1", 1, Yes)?;
    vote_on_proposal(deps.as_mut(), &env, "user2", 1, Yes)?;

    env.block.time = env.block.time.plus_seconds(1001);

    unstake_nfts(deps.as_mut(), &env, "user2", vec!["token2", "token3"])?;
    stake_nfts(
        &mut deps.as_mut(),
        &env,
        NFT_ADDR,
        "user3",
        vec!["token4", "token5", "token6", "token7", "token8", "token9"],
    )?;

    let proposal = query_proposal(
        mock_query_ctx(deps.as_ref(), &env),
        ProposalParams { proposal_id: 1 },
    )?;
    assert_eq!(proposal.total_votes_available, Uint128::from(3u128));

    execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 1 }),
    )?;

    // has to be passed, since at the time of expiration, there were 3 total available votes, 3 cast for yes, and quorum is 50%
    assert_proposal_status(&mock_query_ctx(deps.as_ref(), &env), 1, Passed);

    Ok(())
}

#[ignore]
#[test]
fn execute_proposal_in_multisig_uses_total_votes_available_at_expiration() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("user", &[]);

    env.block.time = Timestamp::from_seconds(12000);
    let dao_gov_config = DaoGovConfig {
        vote_duration: 1000,
        quorum: Decimal::from_ratio(50u8, 100u8),
        threshold: Decimal::from_ratio(50u8, 100u8),
        ..stub_dao_gov_config()
    };

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        multisig_dao_membership_info_with_members(&[
            ("member1", 5u64),
            ("member2", 5u64),
            ("member3", 10u64),
        ]),
        Some(dao_gov_config.clone()),
        None,
    )?;

    // proposal to modify members' weights
    create_proposal(
        deps.as_mut(),
        &env,
        &mock_info("member1", &vec![]),
        None,
        None,
        vec![ModifyMultisigMembership(ModifyMultisigMembershipMsg {
            edit_members: vec![
                MultisigMember {
                    address: "member2".to_string(),
                    weight: Uint128::from(20u64),
                },
                MultisigMember {
                    address: "member3".to_string(),
                    weight: Uint128::from(11u64),
                },
            ],
        })],
    )?;

    env.block.time = env.block.time.plus_seconds(10);

    // actual proposal whose votes are to be tested
    create_stub_proposal(deps.as_mut(), &env, &mock_info("member1", &vec![]))?;

    env.block.time = env.block.time.plus_seconds(10);

    // proposal to modify members' weights that will execute after proposal being tested ends
    create_proposal(
        deps.as_mut(),
        &env,
        &mock_info("member1", &vec![]),
        None,
        None,
        vec![ModifyMultisigMembership(ModifyMultisigMembershipMsg {
            edit_members: vec![MultisigMember {
                address: "member2".to_string(),
                weight: Uint128::zero(),
            }],
        })],
    )?;

    vote_on_proposal(deps.as_mut(), &env, "member3", 1, Yes)?;

    let proposal = query_proposal(
        mock_query_ctx(deps.as_ref(), &env),
        ProposalParams { proposal_id: 1 },
    )?;
    assert_eq!(proposal.total_votes_available, Uint128::from(20u128));

    env.block.time = env.block.time.plus_seconds(981);

    execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 1 }),
    )?;

    vote_on_proposal(deps.as_mut(), &env, "member3", 2, Yes)?;
    vote_on_proposal(deps.as_mut(), &env, "member2", 3, Yes)?;

    env.block.time = env.block.time.plus_seconds(20);

    execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 3 }),
    )?;

    assert_member_voting_power(
        &mock_query_ctx(deps.as_ref(), &env),
        "member2",
        Decimal::zero(),
    );

    let proposal = query_proposal(
        mock_query_ctx(deps.as_ref(), &env),
        ProposalParams { proposal_id: 2 },
    )?;
    assert_eq!(proposal.total_votes_available, Uint128::from(36u128));

    execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteProposal(ExecuteProposalMsg { proposal_id: 2 }),
    )?;

    let proposal = query_proposal(
        mock_query_ctx(deps.as_ref(), &env),
        ProposalParams { proposal_id: 2 },
    )?;
    assert_eq!(proposal.total_votes_available, Uint128::from(36u128));

    // should not pass, since at the time of expiration there were 36 total votes and 11 cast for 'yes' with 50% quorum
    assert_proposal_status(&mock_query_ctx(deps.as_ref(), &env), 2, Rejected);

    Ok(())
}
