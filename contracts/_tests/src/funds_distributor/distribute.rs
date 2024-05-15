use crate::funds_distributor::funds_distributor_helpers::{cast_vote, claim_native_rewards, create_proposal, distribute_native_funds, distribute_native_funds_action, execute_proposal, stake_denom, update_number_proposals_tracked};
use crate::helpers::cw_multitest_helpers::{
    startup_with_versioning, ADMIN, CW20_TOKEN1, ULUNA, USER1, USER2,
};
use crate::helpers::facade_helpers::facade;
use crate::helpers::factory_helpers::{asset_whitelist, create_dao_and_get_addr, default_create_dao_msg, default_gov_config, default_new_token_membership, new_denom_membership, new_multisig_membership, new_token_membership};
use crate::traits::ImplApp;
use cosmwasm_std::{coins, Addr, Uint128};
use cw_asset::AssetInfoUnchecked;
use cw_multi_test::Executor;
use enterprise_factory_api::api::CreateDaoMsg;
use enterprise_governance_controller_api::api::{
    GovConfig, ProposalAction, UpdateAssetWhitelistProposalActionMsg,
};
use funds_distributor_api::api::DistributionType::{Membership, Participation};
use funds_distributor_api::msg::ExecuteMsg::DistributeNative;
use poll_engine_api::api::VoteOutcome::Yes;
use ProposalAction::UpdateAssetWhitelist;

#[test]
fn distribute_total_weight_0_fails() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_token_membership(default_new_token_membership()),
        asset_whitelist: asset_whitelist(vec![ULUNA], vec![CW20_TOKEN1]),
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    let result = distribute_native_funds(&mut app, ADMIN, ULUNA, 1, Participation, dao);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn distribute_participation_without_votes_fails() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 2)]),
        asset_whitelist: asset_whitelist(vec![ULUNA], vec![CW20_TOKEN1]),
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    let result = distribute_native_funds(&mut app, ADMIN, ULUNA, 1, Participation, dao);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn distribute_participation_with_single_proposal_and_no_votes_fails() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 2)]),
        asset_whitelist: asset_whitelist(vec![ULUNA], vec![CW20_TOKEN1]),
        proposals_tracked_for_participation_rewards: Some(1),
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    let result = distribute_native_funds(&mut app, ADMIN, ULUNA, 1, Participation, dao);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn distribute_participation_with_single_proposal_single_vote_distributes_all_rewards() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 2)]),
        asset_whitelist: asset_whitelist(vec![ULUNA], vec![CW20_TOKEN1]),
        proposals_tracked_for_participation_rewards: Some(1),
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;
    cast_vote(&mut app, dao.clone(), USER1, 1, Yes)?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 1, Participation, dao.clone())?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 1)]);

    Ok(())
}

#[test]
fn distribute_participation_with_single_proposal_two_votes_distributes_properly() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 2)]),
        asset_whitelist: asset_whitelist(vec![ULUNA], vec![CW20_TOKEN1]),
        proposals_tracked_for_participation_rewards: Some(1),
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    cast_vote(&mut app, dao.clone(), USER1, 1, Yes)?;

    cast_vote(&mut app, dao.clone(), USER2, 1, Yes)?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 10, Participation, dao.clone())?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 3)]);
    facade(&app, dao)
        .funds_distributor()
        .assert_native_user_rewards(USER2, vec![(ULUNA, 6)]);

    Ok(())
}

#[test]
fn distribute_participation_with_several_proposals_and_two_votes_distributes_properly() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 2)]),
        asset_whitelist: asset_whitelist(vec![ULUNA], vec![CW20_TOKEN1]),
        proposals_tracked_for_participation_rewards: Some(2),
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    for _ in 0..3 {
        create_proposal(&mut app, USER1, dao.clone(), vec![])?;
    }

    cast_vote(&mut app, dao.clone(), USER1, 1, Yes)?;
    cast_vote(&mut app, dao.clone(), USER2, 1, Yes)?;

    cast_vote(&mut app, dao.clone(), USER1, 2, Yes)?;

    cast_vote(&mut app, dao.clone(), USER1, 3, Yes)?;
    cast_vote(&mut app, dao.clone(), USER2, 3, Yes)?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 13, Participation, dao.clone())?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 6)]);
    facade(&app, dao)
        .funds_distributor()
        .assert_native_user_rewards(USER2, vec![(ULUNA, 6)]);

    Ok(())
}

#[test]
fn distribute_participation_with_new_proposal_keeps_working() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 2)]),
        asset_whitelist: asset_whitelist(vec![ULUNA], vec![CW20_TOKEN1]),
        proposals_tracked_for_participation_rewards: Some(2),
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    cast_vote(&mut app, dao.clone(), USER1, 1, Yes)?;

    cast_vote(&mut app, dao.clone(), USER2, 1, Yes)?;

    cast_vote(&mut app, dao.clone(), USER2, 2, Yes)?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 10, Participation, dao.clone())?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 2)]);
    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER2, vec![(ULUNA, 8)]);

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    cast_vote(&mut app, dao.clone(), USER1, 2, Yes)?;
    cast_vote(&mut app, dao.clone(), USER1, 3, Yes)?;

    cast_vote(&mut app, dao.clone(), USER2, 3, Yes)?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 2)]);
    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER2, vec![(ULUNA, 8)]);

    distribute_native_funds(&mut app, ADMIN, ULUNA, 12, Participation, dao.clone())?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 6)]);
    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER2, vec![(ULUNA, 16)]);

    Ok(())
}

#[test]
fn radzion_bug() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1)]),
        asset_whitelist: asset_whitelist(vec![ULUNA], vec![]),
        proposals_tracked_for_participation_rewards: Some(1),
        gov_config: GovConfig {
            allow_early_proposal_execution: true,
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    cast_vote(&mut app, dao.clone(), USER1, 1, Yes)?;

    app.mint_native(vec![(ADMIN, coins(1000, ULUNA))]);
    app.execute_contract(
        Addr::unchecked(ADMIN),
        facade(&app, dao.clone()).funds_distributor_addr(),
        &DistributeNative {
            distribution_type: Some(Participation),
        },
        &coins(1000, ULUNA),
    )?;

    app.mint_native(vec![(
        facade(&app, dao.clone()).treasury_addr().to_string(),
        coins(1000, ULUNA),
    )]);

    create_proposal(
        &mut app,
        USER1,
        dao.clone(),
        vec![distribute_native_funds_action(ULUNA, 1000, Participation)],
    )?;

    cast_vote(&mut app, dao.clone(), USER1, 2, Yes)?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 1000)]);

    execute_proposal(&mut app, USER1, dao.clone(), 2)?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 2000)]);

    Ok(())
}

// TODO: this seems to add no new value to the suite
#[test]
fn radzion_bug2() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1)]),
        asset_whitelist: None,
        proposals_tracked_for_participation_rewards: Some(1),
        gov_config: GovConfig {
            allow_early_proposal_execution: true,
            vote_duration: 120,
            ..default_gov_config()
        },
        minimum_weight_for_rewards: Some(Uint128::one()),
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    // app.mint_native(vec![(USER1, coins(1000, ULUNA))]);
    //
    // let membership = facade(&app, dao.clone()).membership().addr;
    // app.execute_contract(Addr::unchecked(USER1), membership, &denom_staking_api::msg::ExecuteMsg::Stake { user: None }, &coins(1000, ULUNA))?;

    create_proposal(
        &mut app,
        USER1,
        dao.clone(),
        vec![UpdateAssetWhitelist(
            UpdateAssetWhitelistProposalActionMsg {
                remote_treasury_target: None,
                add: vec![AssetInfoUnchecked::native(ULUNA)],
                remove: vec![],
            },
        )],
    )?;

    cast_vote(&mut app, dao.clone(), USER1, 1, Yes)?;

    execute_proposal(&mut app, USER1, dao.clone(), 1)?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 1000, Participation, dao.clone())?;

    app.mint_native(vec![(
        facade(&app, dao.clone()).treasury_addr().to_string(),
        coins(1000, ULUNA),
    )]);

    create_proposal(
        &mut app,
        USER1,
        dao.clone(),
        vec![distribute_native_funds_action(ULUNA, 1000, Participation)],
    )?;

    cast_vote(&mut app, dao.clone(), USER1, 2, Yes)?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 1000)]);

    execute_proposal(&mut app, USER1, dao.clone(), 2)?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 2000)]);

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 1000, Membership, dao.clone())?;
    claim_native_rewards(&mut app, USER1, ULUNA, dao.clone())?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 1000, Membership, dao.clone())?;
    claim_native_rewards(&mut app, USER1, ULUNA, dao.clone())?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 1000, Membership, dao.clone())?;
    claim_native_rewards(&mut app, USER1, ULUNA, dao.clone())?;

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 1000, Membership, dao.clone())?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 1000)]);

    Ok(())
}

#[test]
fn distribute_participation_after_n_updates() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 2)]),
        asset_whitelist: asset_whitelist(vec![ULUNA], vec![]),
        proposals_tracked_for_participation_rewards: Some(2),
        gov_config: GovConfig {
            allow_early_proposal_execution: true,
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    cast_vote(&mut app, dao.clone(), USER1, 1, Yes)?;
    cast_vote(&mut app, dao.clone(), USER2, 1, Yes)?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 3, Participation, dao.clone())?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 1)]);

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER2, vec![(ULUNA, 2)]);

    create_proposal(
        &mut app,
        USER1,
        dao.clone(),
        vec![update_number_proposals_tracked(1)],
    )?;

    cast_vote(&mut app, dao.clone(), USER2, 2, Yes)?;

    execute_proposal(&mut app, USER1, dao.clone(), 2)?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_number_proposals_tracked(1);

    distribute_native_funds(&mut app, ADMIN, ULUNA, 5, Participation, dao.clone())?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 1)]);

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER2, vec![(ULUNA, 7)]);

    Ok(())
}

#[test]
fn distribute_participation_after_new_proposal() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 2)]),
        asset_whitelist: asset_whitelist(vec![ULUNA], vec![]),
        proposals_tracked_for_participation_rewards: Some(2),
        gov_config: GovConfig {
            allow_early_proposal_execution: true,
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    cast_vote(&mut app, dao.clone(), USER1, 1, Yes)?;
    cast_vote(&mut app, dao.clone(), USER2, 1, Yes)?;

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    cast_vote(&mut app, dao.clone(), USER1, 2, Yes)?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 4, Participation, dao.clone())?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 2)]);

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER2, vec![(ULUNA, 2)]);

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 5, Participation, dao.clone())?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 7)]);

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER2, vec![(ULUNA, 2)]);

    Ok(())
}

#[test]
fn distribute_participation_after_user_weight_change() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_denom_membership(ULUNA, 300),
        asset_whitelist: asset_whitelist(vec![ULUNA], vec![]),
        proposals_tracked_for_participation_rewards: Some(2),
        gov_config: GovConfig {
            allow_early_proposal_execution: true,
            vote_duration: 300,
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    stake_denom(&mut app, USER1, dao.clone(), ULUNA, 1)?;

    stake_denom(&mut app, USER2, dao.clone(), ULUNA, 2)?;

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    cast_vote(&mut app, dao.clone(), USER1, 1, Yes)?;
    cast_vote(&mut app, dao.clone(), USER2, 1, Yes)?;

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    cast_vote(&mut app, dao.clone(), USER1, 2, Yes)?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 4, Participation, dao.clone())?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 2)]);

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER2, vec![(ULUNA, 2)]);

    stake_denom(&mut app, USER1, dao.clone(), ULUNA, 4)?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 12, Participation, dao.clone())?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 12)]);

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER2, vec![(ULUNA, 4)]);

    Ok(())
}
