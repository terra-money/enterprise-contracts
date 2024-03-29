use crate::helpers::cw_multitest_helpers::{
    startup_with_versioning, ADMIN, CODE_ID_CW20, CODE_ID_CW3, USER1, USER2, USER3,
};
use crate::helpers::facade_helpers::{from_facade_gov_config, TestFacade};
use crate::helpers::factory_helpers::{
    create_dao, default_create_dao_msg, default_dao_council, default_dao_metadata,
    default_gov_config, get_first_dao, import_cw3_membership, new_multisig_membership,
};
use crate::traits::IntoAddr;
use cosmwasm_std::Decimal;
use cw3_fixed_multisig::msg::Voter;
use cw_multi_test::Executor;
use cw_utils::{Duration, Threshold};
use enterprise_facade_api::api::DaoType::Multisig;
use enterprise_facade_common::facade::EnterpriseFacade;
use enterprise_factory_api::api::CreateDaoMsg;
use enterprise_governance_controller_api::api::GovConfig;

#[test]
fn create_new_multisig_dao() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_metadata: default_dao_metadata(),
        gov_config: default_gov_config(),
        dao_council: Some(default_dao_council()),
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 2), (USER3, 5)]),
        asset_whitelist: None,
        nft_whitelist: None,
        minimum_weight_for_rewards: None,
        proposals_tracked_for_participation_rewards: None,
        cross_chain_treasuries: None,
    };

    create_dao(&mut app, msg)?;

    let dao_addr = get_first_dao(&app)?;
    let facade = TestFacade {
        app: &app,
        dao_addr,
    };

    assert_eq!(
        facade.member_info(USER1)?.voting_power,
        Decimal::from_ratio(1u8, 8u8)
    );

    assert_eq!(
        facade.member_info(USER2)?.voting_power,
        Decimal::from_ratio(2u8, 8u8)
    );

    assert_eq!(
        facade.member_info(USER3)?.voting_power,
        Decimal::from_ratio(5u8, 8u8)
    );

    facade.assert_multisig_members_list(None, Some(1), vec![(USER1, 1)]);
    facade.assert_multisig_members_list(Some(USER1), None, vec![(USER2, 2), (USER3, 5)]);

    facade.assert_total_staked(0);

    facade
        .membership()
        .assert_user_weights(vec![(USER1, 1), (USER2, 2), (USER3, 5)]);
    facade.membership().assert_total_weight(8);

    facade.assert_dao_type(Multisig);

    Ok(())
}

#[test]
fn create_new_multisig_dao_with_minimum_deposit_fails() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        gov_config: GovConfig {
            minimum_deposit: Some(2u8.into()),
            ..default_gov_config()
        },
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 2), (USER3, 5)]),
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn import_cw3_dao() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let cw3_threshold = Decimal::percent(79);
    let cw3_voting_time = 10_000u64;
    let cw3_contract = app
        .instantiate_contract(
            CODE_ID_CW3,
            ADMIN.into_addr(),
            &cw3_fixed_multisig::msg::InstantiateMsg {
                voters: vec![
                    Voter {
                        addr: USER1.to_string(),
                        weight: 10,
                    },
                    Voter {
                        addr: USER3.to_string(),
                        weight: 90,
                    },
                ],
                threshold: Threshold::AbsolutePercentage {
                    percentage: cw3_threshold,
                },
                max_voting_period: Duration::Time(cw3_voting_time),
            },
            &[],
            "CW3 multisig",
            Some(ADMIN.to_string()),
        )
        .unwrap();

    let gov_config = GovConfig {
        quorum: Decimal::percent(10),
        threshold: Decimal::percent(66),
        veto_threshold: Some(Decimal::percent(50)),
        vote_duration: 100,
        minimum_deposit: None,
        allow_early_proposal_execution: false,
    };
    let msg = CreateDaoMsg {
        dao_metadata: default_dao_metadata(),
        gov_config: gov_config.clone(),
        dao_council: Some(default_dao_council()),
        dao_membership: import_cw3_membership(cw3_contract),
        asset_whitelist: None,
        nft_whitelist: None,
        minimum_weight_for_rewards: None,
        proposals_tracked_for_participation_rewards: None,
        cross_chain_treasuries: None,
    };

    create_dao(&mut app, msg)?;

    let dao_addr = get_first_dao(&app)?;
    let facade = TestFacade {
        app: &app,
        dao_addr,
    };

    assert_eq!(
        facade.member_info(USER1)?.voting_power,
        Decimal::percent(10)
    );

    assert_eq!(facade.member_info(USER2)?.voting_power, Decimal::zero());

    assert_eq!(
        facade.member_info(USER3)?.voting_power,
        Decimal::percent(90)
    );

    // TODO: this bunch of steps for verifying member weights in all places is copied over, extract it to a helper
    facade.assert_multisig_members_list(None, Some(1), vec![(USER1, 10)]);
    facade.assert_multisig_members_list(Some(USER1), None, vec![(USER3, 90)]);

    facade.assert_total_staked(0);

    facade
        .membership()
        .assert_user_weights(vec![(USER1, 10), (USER2, 0), (USER3, 90)]);
    facade.membership().assert_total_weight(100);

    // assert that we set different governance params from the ones in CW3, and ours are the ones used in the DAO
    assert_ne!(cw3_voting_time, gov_config.vote_duration);
    assert_ne!(cw3_threshold, gov_config.threshold);
    assert_eq!(
        from_facade_gov_config(facade.query_dao_info()?.gov_config),
        gov_config
    );

    facade.assert_dao_type(Multisig);

    Ok(())
}

#[test]
fn import_cw3_dao_with_minimum_deposit_fails() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let cw3_contract = app
        .instantiate_contract(
            CODE_ID_CW3,
            ADMIN.into_addr(),
            &cw3_fixed_multisig::msg::InstantiateMsg {
                voters: vec![Voter {
                    addr: USER1.to_string(),
                    weight: 10,
                }],
                threshold: Threshold::AbsolutePercentage {
                    percentage: Decimal::percent(79),
                },
                max_voting_period: Duration::Time(10_000u64),
            },
            &[],
            "CW3 multisig",
            Some(ADMIN.to_string()),
        )
        .unwrap();

    let msg = CreateDaoMsg {
        gov_config: GovConfig {
            minimum_deposit: Some(2u8.into()),
            ..default_gov_config()
        },
        dao_membership: import_cw3_membership(cw3_contract),
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn import_non_cw3_dao_fails() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let cw20_contract = app
        .instantiate_contract(
            CODE_ID_CW20,
            ADMIN.into_addr(),
            &cw20_base::msg::InstantiateMsg {
                name: "CW20 token".to_string(),
                symbol: "TKN".to_string(),
                decimals: 6,
                initial_balances: vec![],
                mint: None,
                marketing: None,
            },
            &[],
            "CW20",
            Some(ADMIN.to_string()),
        )
        .unwrap();

    let msg = CreateDaoMsg {
        dao_metadata: default_dao_metadata(),
        gov_config: default_gov_config(),
        dao_council: Some(default_dao_council()),
        dao_membership: import_cw3_membership(cw20_contract),
        asset_whitelist: None,
        nft_whitelist: None,
        minimum_weight_for_rewards: None,
        proposals_tracked_for_participation_rewards: None,
        cross_chain_treasuries: None,
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}
