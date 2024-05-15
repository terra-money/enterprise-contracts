use crate::funds_distributor::funds_distributor_helpers::{
    cast_vote, create_proposal, distribute_native_funds, execute_proposal,
    update_minimum_weight_for_rewards, update_number_proposals_tracked,
};
use crate::helpers::cw_multitest_helpers::{startup_with_versioning, ADMIN, ULUNA, USER1, USER2};
use crate::helpers::facade_helpers::facade;
use crate::helpers::factory_helpers::{
    asset_whitelist, create_dao_and_get_addr, default_create_dao_msg, default_gov_config,
    new_multisig_membership,
};
use crate::helpers::funds_distributor_helpers::FundsDistributorContract;
use cosmwasm_std::Uint128;
use enterprise_factory_api::api::CreateDaoMsg;
use enterprise_governance_controller_api::api::GovConfig;
use funds_distributor_api::api::DistributionType::Membership;
use poll_engine_api::api::VoteOutcome::Yes;

#[test]
fn minimum_weight_for_rewards_changes_properly() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 3), (USER2, 1)]),
        asset_whitelist: asset_whitelist(vec![ULUNA], vec![]),
        minimum_weight_for_rewards: Some(Uint128::zero()),
        gov_config: GovConfig {
            allow_early_proposal_execution: true,
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    distribute_native_funds(&mut app, ADMIN, ULUNA, 4, Membership, dao.clone())?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 3)]);
    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER2, vec![(ULUNA, 1)]);

    create_proposal(
        &mut app,
        USER1,
        dao.clone(),
        vec![update_minimum_weight_for_rewards(3)],
    )?;

    cast_vote(&mut app, dao.clone(), USER1, 1, Yes)?;

    execute_proposal(&mut app, USER1, dao.clone(), 1)?;

    let minimum_eligible_weight = facade(&app, dao.clone())
        .funds_distributor()
        .minimum_eligible_weight()?;

    assert_eq!(minimum_eligible_weight.u128(), 3u128,);

    distribute_native_funds(&mut app, ADMIN, ULUNA, 3, Membership, dao.clone())?;

    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER1, vec![(ULUNA, 6)]);
    facade(&app, dao.clone())
        .funds_distributor()
        .assert_native_user_rewards(USER2, vec![(ULUNA, 1)]);

    Ok(())
}
