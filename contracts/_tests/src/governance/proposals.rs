use crate::governance::governance_helpers::{
    cast_vote, create_proposal, execute_proposal, query_proposal,
};
use crate::helpers::cw_multitest_helpers::{
    increase_time_block, startup_with_versioning, ADMIN, USER1, USER2, USER3,
};
use crate::helpers::factory_helpers::{
    create_dao_and_get_addr, default_create_dao_msg, default_gov_config, new_multisig_membership,
};
use cosmwasm_std::Decimal;
use enterprise_factory_api::api::CreateDaoMsg;
use enterprise_governance_controller_api::api::{GovConfig, ProposalStatus};
use poll_engine_api::api::VoteOutcome::Yes;

#[test]
fn simple_text_proposal_can_pass_and_execute() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 3)]),
        gov_config: GovConfig {
            quorum: Decimal::percent(25),
            allow_early_proposal_execution: true,
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    cast_vote(&mut app, dao.clone(), USER1, 1, Yes)?;

    execute_proposal(&mut app, ADMIN, dao.clone(), 1)?;

    let proposal = query_proposal(&app, dao, 1)?;
    assert_eq!(proposal.proposal_status, ProposalStatus::Executed);

    Ok(())
}

#[test]
fn proposals_without_enough_votes_cannot_be_executed() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 3)]),
        gov_config: GovConfig {
            quorum: Decimal::percent(26),
            allow_early_proposal_execution: true,
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    cast_vote(&mut app, dao.clone(), USER1, 1, Yes)?;

    let result = execute_proposal(&mut app, ADMIN, dao.clone(), 1);

    assert!(result.is_err());

    let proposal = query_proposal(&app, dao, 1)?;
    assert_eq!(proposal.proposal_status, ProposalStatus::InProgress);

    Ok(())
}

#[test]
fn cannot_vote_post_proposal_expiration() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 3)]),
        gov_config: GovConfig {
            vote_duration: 100,
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    increase_time_block(&mut app, 101);

    let result = cast_vote(&mut app, dao.clone(), USER2, 1, Yes);

    assert!(result.is_err());

    let proposal = query_proposal(&app, dao, 1)?;
    assert_eq!(proposal.proposal_status, ProposalStatus::Rejected);

    Ok(())
}

#[test]
fn cannot_execute_early_if_flag_disabled() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 3)]),
        gov_config: GovConfig {
            allow_early_proposal_execution: false,
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    create_proposal(&mut app, USER1, dao.clone(), vec![])?;

    cast_vote(&mut app, dao.clone(), USER1, 1, Yes)?;
    cast_vote(&mut app, dao.clone(), USER2, 1, Yes)?;

    let proposal = query_proposal(&app, dao, 1)?;
    assert_eq!(proposal.proposal_status, ProposalStatus::InProgress);

    Ok(())
}

#[test]
fn cannot_submit_proposal_if_not_member() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 3)]),
        gov_config: GovConfig {
            allow_early_proposal_execution: false,
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    let result = create_proposal(&mut app, USER3, dao.clone(), vec![]);

    assert!(result.is_err());

    Ok(())
}
