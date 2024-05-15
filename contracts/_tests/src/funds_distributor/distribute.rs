use crate::helpers::cw_multitest_helpers::{
    startup_with_versioning, ADMIN, CW20_TOKEN1, ULUNA, USER1, USER2,
};
use crate::helpers::facade_helpers::facade;
use crate::helpers::factory_helpers::{asset_whitelist, create_dao_and_get_addr, default_create_dao_msg, default_gov_config, default_new_token_membership, new_denom_membership, new_multisig_membership, new_token_membership};
use crate::traits::ImplApp;
use cosmwasm_std::{coins, Addr, Uint128};
use cw_asset::{AssetInfoUnchecked, AssetUnchecked};
use cw_multi_test::{App, AppResponse, Executor};
use enterprise_facade_api::api::{NumberProposalsTrackedResponse, ProposalId};
use enterprise_factory_api::api::CreateDaoMsg;
use enterprise_governance_controller_api::api::{CastVoteMsg, CreateProposalMsg, DistributeFundsMsg, ExecuteProposalMsg, GovConfig, ProposalAction, UpdateAssetWhitelistProposalActionMsg, UpdateNumberProposalsTrackedMsg};
use enterprise_governance_controller_api::msg::ExecuteMsg::{
    CastVote, CreateProposal, ExecuteProposal,
};
use funds_distributor_api::api::{ClaimRewardsMsg, DistributionType};
use funds_distributor_api::api::DistributionType::{Membership, Participation};
use funds_distributor_api::msg::ExecuteMsg::{ClaimRewards, DistributeNative};
use funds_distributor_api::msg::QueryMsg::NumberProposalsTracked;
use poll_engine_api::api::VoteOutcome;
use poll_engine_api::api::VoteOutcome::Yes;
use ProposalAction::{DistributeFunds, UpdateAssetWhitelist, UpdateNumberProposalsTracked};

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

    app.execute_contract(
        Addr::unchecked(USER1),
        facade(&app, dao.clone()).gov_controller_addr(),
        &CreateProposal(CreateProposalMsg {
            title: "sth".to_string(),
            description: None,
            proposal_actions: vec![],
            deposit_owner: None,
        }),
        &vec![],
    )?;

    app.execute_contract(
        Addr::unchecked(USER1),
        facade(&app, dao.clone()).gov_controller_addr(),
        &CreateProposal(CreateProposalMsg {
            title: "sth".to_string(),
            description: None,
            proposal_actions: vec![],
            deposit_owner: None,
        }),
        &vec![],
    )?;

    app.execute_contract(
        Addr::unchecked(USER1),
        facade(&app, dao.clone()).gov_controller_addr(),
        &CastVote(CastVoteMsg {
            proposal_id: 1,
            outcome: VoteOutcome::Yes,
        }),
        &vec![],
    )?;

    app.execute_contract(
        Addr::unchecked(USER2),
        facade(&app, dao.clone()).gov_controller_addr(),
        &CastVote(CastVoteMsg {
            proposal_id: 1,
            outcome: VoteOutcome::Yes,
        }),
        &vec![],
    )?;

    app.execute_contract(
        Addr::unchecked(USER2),
        facade(&app, dao.clone()).gov_controller_addr(),
        &CastVote(CastVoteMsg {
            proposal_id: 2,
            outcome: VoteOutcome::Yes,
        }),
        &vec![],
    )?;

    // TODO: use helper instead of this
    app.mint_native(vec![(ADMIN, coins(10, ULUNA))]);
    app.execute_contract(
        Addr::unchecked(ADMIN),
        facade(&app, dao.clone()).funds_distributor_addr(),
        &DistributeNative {
            distribution_type: Some(Participation),
        },
        &coins(10, ULUNA),
    )?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 2)]);
    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER2, vec![(ULUNA, 8)]);

    app.execute_contract(
        Addr::unchecked(USER1),
        facade(&app, dao.clone()).gov_controller_addr(),
        &CreateProposal(CreateProposalMsg {
            title: "sth".to_string(),
            description: None,
            proposal_actions: vec![],
            deposit_owner: None,
        }),
        &vec![],
    )?;

    app.execute_contract(
        Addr::unchecked(USER1),
        facade(&app, dao.clone()).gov_controller_addr(),
        &CastVote(CastVoteMsg {
            proposal_id: 2,
            outcome: VoteOutcome::Yes,
        }),
        &vec![],
    )?;

    app.execute_contract(
        Addr::unchecked(USER1),
        facade(&app, dao.clone()).gov_controller_addr(),
        &CastVote(CastVoteMsg {
            proposal_id: 3,
            outcome: VoteOutcome::Yes,
        }),
        &vec![],
    )?;

    app.execute_contract(
        Addr::unchecked(USER2),
        facade(&app, dao.clone()).gov_controller_addr(),
        &CastVote(CastVoteMsg {
            proposal_id: 3,
            outcome: VoteOutcome::Yes,
        }),
        &vec![],
    )?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 2)]);
    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER2, vec![(ULUNA, 8)]);

    // TODO: use helper instead of this
    app.mint_native(vec![(ADMIN, coins(12, ULUNA))]);
    app.execute_contract(
        Addr::unchecked(ADMIN),
        facade(&app, dao.clone()).funds_distributor_addr(),
        &DistributeNative {
            distribution_type: Some(Participation),
        },
        &coins(12, ULUNA),
    )?;

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

    create_proposal(&mut app, USER1, dao.clone(), vec![
        UpdateAssetWhitelist(
            UpdateAssetWhitelistProposalActionMsg {
                remote_treasury_target: None,
                add: vec![AssetInfoUnchecked::native(ULUNA)],
                remove: vec![],
            }
        ),
    ])?;

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

    create_proposal(
        &mut app,
        USER1,
        dao.clone(),
        vec![],
    )?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 1000, Membership, dao.clone())?;
    claim_native_rewards(&mut app, USER1, ULUNA, dao.clone())?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 1000, Membership, dao.clone())?;
    claim_native_rewards(&mut app, USER1, ULUNA, dao.clone())?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 1000, Membership, dao.clone())?;
    claim_native_rewards(&mut app, USER1, ULUNA, dao.clone())?;

    create_proposal(
        &mut app,
        USER1,
        dao.clone(),
        vec![],
    )?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 1000, Membership, dao.clone())?;

    facade(&app, dao.clone()).funds_distributor().assert_native_user_rewards(
        USER1,
        vec![(ULUNA, 1000)],
    );

    Ok(())
}

#[test]
fn update_n_bug() -> anyhow::Result<()> {
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

    create_proposal(
        &mut app,
        USER1,
        dao.clone(),
        vec![update_number_proposals_tracked(3)],
    )?;

    cast_vote(&mut app, dao.clone(), USER1, 1, Yes)?;

    execute_proposal(&mut app, USER1, dao.clone(), 1)?;

    let funds = facade(&app, dao.clone())
        .funds_distributor()
        .addr
        .to_string();

    let minimum_weight_for_rewards: NumberProposalsTrackedResponse = app
        .wrap()
        .query_wasm_smart(funds, &NumberProposalsTracked {})?;

    assert_eq!(
        minimum_weight_for_rewards.number_proposals_tracked,
        Some(3u8)
    );

    Ok(())
}

// TODO: move to GovControllerContract trait somehow
fn create_proposal(
    app: &mut App,
    proposer: &str,
    dao: Addr,
    proposal_actions: Vec<ProposalAction>,
) -> anyhow::Result<()> {
    app.execute_contract(
        Addr::unchecked(proposer),
        facade(&app, dao).gov_controller_addr(),
        &CreateProposal(CreateProposalMsg {
            title: "sth".to_string(),
            description: None,
            proposal_actions,
            deposit_owner: None,
        }),
        &vec![],
    )?;
    Ok(())
}

// TODO: move to GovControllerContract trait somehow
fn cast_vote(
    app: &mut App,
    dao: Addr,
    voter: &str,
    proposal_id: ProposalId,
    outcome: VoteOutcome,
) -> anyhow::Result<()> {
    app.execute_contract(
        Addr::unchecked(voter),
        facade(&app, dao).gov_controller_addr(),
        &CastVote(CastVoteMsg {
            proposal_id,
            outcome,
        }),
        &vec![],
    )?;
    Ok(())
}

// TODO: move to GovControllerContract trait somehow
fn execute_proposal(
    app: &mut App,
    executor: &str,
    dao: Addr,
    proposal_id: ProposalId,
) -> anyhow::Result<()> {
    app.execute_contract(
        Addr::unchecked(executor),
        facade(&app, dao).gov_controller_addr(),
        &ExecuteProposal(ExecuteProposalMsg { proposal_id }),
        &vec![],
    )?;
    Ok(())
}

// TODO: move to gov controller helpers
fn update_number_proposals_tracked(n: u8) -> ProposalAction {
    UpdateNumberProposalsTracked(UpdateNumberProposalsTrackedMsg {
        number_proposals_tracked: n,
    })
}

// TODO: move to gov controller helpers
fn distribute_native_funds_action(
    denom: &str,
    amount: u128,
    distribution_type: DistributionType,
) -> ProposalAction {
    DistributeFunds(DistributeFundsMsg {
        funds: vec![AssetUnchecked::native(denom, Uint128::from(amount))],
        distribution_type: Some(distribution_type),
    })
}

fn distribute_native_funds(
    app: &mut App,
    distributor: &str,
    denom: &str,
    amount: u128,
    distribution_type: DistributionType,
    dao: Addr,
) -> anyhow::Result<AppResponse> {
    app.mint_native(vec![(distributor, coins(amount, denom))]);
    app.execute_contract(
        Addr::unchecked(distributor),
        facade(&app, dao).funds_distributor().addr,
        &DistributeNative {
            distribution_type: Some(distribution_type),
        },
        &coins(amount, denom),
    )
}

fn claim_native_rewards(
    app: &mut App,
    user: &str,
    denom: &str,
    dao: Addr,
) -> anyhow::Result<AppResponse> {
    app.execute_contract(
        Addr::unchecked(user),
        facade(&app, dao).funds_distributor().addr,
        &ClaimRewards(ClaimRewardsMsg {
            user: USER1.to_string(),
            native_denoms: vec![denom.to_string()],
            cw20_assets: vec![],
        }),
        &vec![],
    )
}
