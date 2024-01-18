use common::commons::ModifyValue;

use cosmwasm_std::Coin;
use cw_asset::{AssetBase, AssetInfoBase};
use cw_utils::Duration;
use enterprise_factory_api::api::{CreateDaoMembershipMsg, NewDenomMembershipMsg};
use enterprise_governance_controller_api::api::{
    ProposalAction, ProposalStatus, UpdateAssetWhitelistProposalActionMsg,
};
use enterprise_protocol::api::{DaoType, UpdateMetadataMsg};

use poll_engine_api::api::VoteOutcome;

use crate::{
    helpers::{
        create_standard_msg_new_dao, increase_time_block, qy_all_proposals, qy_cw20_balance,
        qy_get_all_contracts_of_a_dao, qy_get_dao_type, qy_get_membership_unerline_from_dao_type,
        qy_membership_total_weight, qy_membership_user_weight, qy_multisig_total_weight,
        qy_multisig_user_weight, qy_proposal, run_create_council_proposal, run_create_gov_proposal,
        run_execute_proposal, run_membership_claim_cw20, run_membership_deposit,
        run_membership_unstake_cw20, run_vote_council_proposal, run_vote_gov_proposal,
        startup_custom_dao, startup_default_dao, COUNCIL, INITIAL_USERS_BALANCE_EACH,
        MINIMUM_DEPOSIT, UNLOCKING_PERIOD_GOV, UNLOCKING_PERIOD_MEMBERSHIP, USERS, VOTE_DURATION,
    },
    traits::{ImplApp, IntoUint, IntoVoteResult},
};

#[test]
fn test() {
    let app = startup_default_dao(true);

    let contracts = qy_get_all_contracts_of_a_dao(&app, 1);

    let dao_type = qy_get_dao_type(&app, contracts.clone().enterprise);

    let cw20_address = qy_get_membership_unerline_from_dao_type(&app, &dao_type, &contracts);

    let council_1 = COUNCIL[0];

    let _council_1_balance = qy_cw20_balance(&app, &cw20_address, council_1);

    // let res = run_membership_deposit_cw20(&mut app, council_1, 100_u128.into_uint128(), contracts).unwrap();

    let council_1_weight = qy_multisig_user_weight(
        &app,
        contracts.others.council_membership_contract.clone(),
        council_1,
    );

    let total_weight =
        qy_multisig_total_weight(&app, contracts.others.council_membership_contract.clone());
    assert_eq!(council_1_weight, 1.into_uint128());

    assert_eq!(total_weight, COUNCIL.len().into_uint128());

    let mut app = startup_default_dao(false);

    let action = ProposalAction::UpdateMetadata(UpdateMetadataMsg {
        name: ModifyValue::Change("new_name".to_string()),
        description: ModifyValue::NoChange,
        logo: ModifyValue::NoChange,
        github_username: ModifyValue::NoChange,
        discord_username: ModifyValue::NoChange,
        twitter_username: ModifyValue::NoChange,
        telegram_username: ModifyValue::NoChange,
    });

    run_create_council_proposal(
        &mut app,
        contracts.clone(),
        council_1,
        "title",
        Some("description"),
        vec![action],
    )
    .unwrap();

    run_vote_council_proposal(&mut app, contracts.clone(), COUNCIL[0], 1, VoteOutcome::Yes)
        .unwrap();

    let _status = qy_proposal(&app, contracts.clone(), 1);

    // println!("{status:#?}");

    increase_time_block(&mut app, VOTE_DURATION + 1);

    run_execute_proposal(&mut app, contracts.clone(), COUNCIL[0], 1).unwrap();

    let status = qy_proposal(&app, contracts.clone(), 1);

    assert_eq!(status.proposal_status, ProposalStatus::Executed);

    // println!("{status:#?}");

    run_vote_council_proposal(&mut app, contracts.clone(), COUNCIL[1], 1, VoteOutcome::Yes)
        .unwrap_err();

    run_execute_proposal(&mut app, contracts.clone(), COUNCIL[0], 1).unwrap_err();
}

#[test]
fn test_token() {
    let mut app = startup_default_dao(false);

    let contracts = qy_get_all_contracts_of_a_dao(&app, 1);

    let dao_type = qy_get_dao_type(&app, contracts.clone().enterprise);

    let cw20_address = qy_get_membership_unerline_from_dao_type(&app, &dao_type, &contracts);

    assert_eq!(
        qy_cw20_balance(&app, &cw20_address, USERS[0]),
        INITIAL_USERS_BALANCE_EACH.into_uint128()
    );

    let deposit_amount = 500_u128;

    run_membership_deposit(&mut app, USERS[0], deposit_amount, &contracts).unwrap();
    run_membership_deposit(&mut app, USERS[1], deposit_amount, &contracts).unwrap();

    assert_eq!(
        qy_cw20_balance(&app, &cw20_address, USERS[0]),
        (INITIAL_USERS_BALANCE_EACH - deposit_amount).into_uint128()
    );
    assert_eq!(
        qy_cw20_balance(&app, &cw20_address, USERS[1]),
        (INITIAL_USERS_BALANCE_EACH - deposit_amount).into_uint128()
    );

    assert_eq!(
        qy_membership_user_weight(&app, &contracts, USERS[0]),
        deposit_amount.into_uint128()
    );
    assert_eq!(
        qy_membership_user_weight(&app, &contracts, USERS[1]),
        deposit_amount.into_uint128()
    );

    assert_eq!(
        qy_membership_total_weight(&app, &contracts, None),
        (deposit_amount * 2).into_uint128()
    );

    // assert_eq!(qy_funds_distributor_user_weight(& app, &contracts, USERS[0]), (deposit_amount.into_uint128(), deposit_amount.into_uint128()));
    // assert_eq!(qy_funds_distributor_user_weight(& app, &contracts, USERS[1]), (deposit_amount.into_uint128(), deposit_amount.into_uint128()));

    // assert_eq!(qy_funds_distributor_total_weight(& app, &contracts), (deposit_amount *2).into_uint128() );

    let withdraw_amount = 300_u128;

    run_membership_unstake_cw20(&mut app, USERS[0], withdraw_amount, &contracts).unwrap();

    assert_eq!(
        qy_membership_user_weight(&app, &contracts, USERS[0]),
        (deposit_amount - withdraw_amount).into_uint128()
    );

    // assert_eq!(qy_funds_distributor_user_weight(& app, &contracts, USERS[0]), ((deposit_amount - withdraw_amount).into_uint128(), (deposit_amount - withdraw_amount).into_uint128()));
    // assert_eq!(qy_funds_distributor_total_weight(& app, &contracts), (deposit_amount *2 - withdraw_amount).into_uint128() );

    run_membership_claim_cw20(&mut app, USERS[0], &contracts).unwrap();
    assert_eq!(
        qy_cw20_balance(&app, &cw20_address, USERS[0]),
        (INITIAL_USERS_BALANCE_EACH - deposit_amount).into_uint128()
    );

    increase_time_block(&mut app, UNLOCKING_PERIOD_MEMBERSHIP - 1);
    run_membership_claim_cw20(&mut app, USERS[0], &contracts).unwrap();
    assert_eq!(
        qy_cw20_balance(&app, &cw20_address, USERS[0]),
        (INITIAL_USERS_BALANCE_EACH - deposit_amount).into_uint128()
    );

    increase_time_block(&mut app, 2);
    run_membership_claim_cw20(&mut app, USERS[0], &contracts).unwrap();
    assert_eq!(
        qy_cw20_balance(&app, &cw20_address, USERS[0]),
        (INITIAL_USERS_BALANCE_EACH - deposit_amount + withdraw_amount).into_uint128()
    );

    // Create Proposal UPDATE WHITELIST ASSET

    let asset_name = "uluna";

    let action = ProposalAction::UpdateAssetWhitelist(UpdateAssetWhitelistProposalActionMsg {
        remote_treasury_target: None,
        add: vec![AssetInfoBase::native(asset_name)],
        remove: vec![],
    });

    run_create_gov_proposal(
        &mut app,
        &contracts,
        USERS[0],
        "withelist",
        None,
        vec![action.clone()],
        None,
    )
    .unwrap_err();
    run_create_gov_proposal(
        &mut app,
        &contracts,
        USERS[0],
        "withelist",
        None,
        vec![action],
        Some(AssetBase::new(
            AssetInfoBase::Cw20(cw20_address.clone()),
            MINIMUM_DEPOSIT,
        )),
    )
    .unwrap();

    let id = qy_all_proposals(&app, &contracts)
        .last()
        .unwrap()
        .proposal
        .id;

    run_vote_gov_proposal(&mut app, &contracts, USERS[0], id, VoteOutcome::Yes).unwrap();

    let binding = qy_all_proposals(&app, &contracts);
    let results = binding.last().unwrap().results.clone();

    assert_eq!(
        results.yes(),
        qy_membership_user_weight(&app, &contracts, USERS[0]).u128()
    );

    run_vote_gov_proposal(&mut app, &contracts, USERS[0], id, VoteOutcome::Yes).unwrap();

    let binding = qy_all_proposals(&app, &contracts);
    let results = binding.last().unwrap().results.clone();

    assert_eq!(
        results.yes(),
        qy_membership_user_weight(&app, &contracts, USERS[0]).u128()
    );

    run_vote_gov_proposal(&mut app, &contracts, USERS[1], id, VoteOutcome::No).unwrap();

    let binding = qy_all_proposals(&app, &contracts);
    let results = binding.last().unwrap().results.clone();

    assert_eq!(
        results.yes(),
        qy_membership_user_weight(&app, &contracts, USERS[0]).u128()
    );
    assert_eq!(
        results.no(),
        qy_membership_user_weight(&app, &contracts, USERS[1]).u128()
    );

    let new_withdraw_amount = 100_u128;

    run_membership_unstake_cw20(&mut app, USERS[0], new_withdraw_amount, &contracts).unwrap();
    increase_time_block(&mut app, UNLOCKING_PERIOD_MEMBERSHIP + 1);

    let binding = qy_all_proposals(&app, &contracts);
    let results = binding.last().unwrap().results.clone();

    assert_eq!(
        results.yes(),
        qy_membership_user_weight(&app, &contracts, USERS[0]).u128()
    );

    run_membership_claim_cw20(&mut app, USERS[0], &contracts).unwrap();
    assert_eq!(
        qy_cw20_balance(&app, &cw20_address, USERS[0]),
        (INITIAL_USERS_BALANCE_EACH - deposit_amount + withdraw_amount + new_withdraw_amount
            - MINIMUM_DEPOSIT)
            .into_uint128()
    );

    let binding = qy_all_proposals(&app, &contracts);
    let results = binding.last().unwrap().results.clone();

    assert_eq!(
        results.yes(),
        qy_membership_user_weight(&app, &contracts, USERS[0]).u128()
    );

    // println!("{}",results.yes());
    // println!("{}",qy_membership_user_weight(& app, &contracts, USERS[0]).u128())
}

#[test]
fn test_minimum_deposit_on_denom_membership() {
    let mut msg = create_standard_msg_new_dao(false);

    let denom = "uluna";

    msg.dao_membership = CreateDaoMembershipMsg::NewDenom(NewDenomMembershipMsg {
        denom: "uluna".to_string(),
        unlocking_period: Duration::Time(UNLOCKING_PERIOD_GOV),
    });

    let mut app = startup_custom_dao(msg);

    let contracts = qy_get_all_contracts_of_a_dao(&app, 1);

    // Deposit some tokens

    let mint_amount = 1_000_u128;

    app.mint_native(vec![
        (USERS[0], vec![Coin::new(mint_amount, denom)]),
        (USERS[1], vec![Coin::new(mint_amount, denom)]),
    ]);

    assert_eq!(
        app.wrap().query_balance(USERS[0], denom).unwrap().amount,
        mint_amount.into_uint128()
    );

    let deposit_amount = 500_u128;

    run_membership_deposit(&mut app, USERS[0], deposit_amount, &contracts).unwrap();
    assert_eq!(
        qy_membership_user_weight(&app, &contracts, USERS[0]),
        (deposit_amount).into_uint128()
    );
    assert_eq!(
        qy_membership_total_weight(&app, &contracts, None),
        (deposit_amount).into_uint128()
    );

    run_membership_deposit(&mut app, USERS[1], deposit_amount, &contracts).unwrap();
    assert_eq!(
        qy_membership_user_weight(&app, &contracts, USERS[1]),
        (deposit_amount).into_uint128()
    );
    assert_eq!(
        qy_membership_total_weight(&app, &contracts, None),
        (deposit_amount * 2).into_uint128()
    );

    let native_denom = qy_get_membership_unerline_from_dao_type(&app, &DaoType::Denom, &contracts);

    let new_denom = "uatom";

    let action = ProposalAction::UpdateAssetWhitelist(UpdateAssetWhitelistProposalActionMsg {
        remote_treasury_target: None,
        add: vec![AssetInfoBase::native(new_denom)],
        remove: vec![],
    });

    run_create_gov_proposal(
        &mut app,
        &contracts,
        USERS[0],
        "withelist",
        None,
        vec![action.clone()],
        None,
    )
    .unwrap_err();
    run_create_gov_proposal(
        &mut app,
        &contracts,
        USERS[0],
        "withelist",
        None,
        vec![action],
        Some(AssetBase::new(
            AssetInfoBase::<String>::Native(native_denom),
            MINIMUM_DEPOSIT,
        )),
    )
    .unwrap();

    let id = qy_all_proposals(&app, &contracts)
        .last()
        .unwrap()
        .proposal
        .id;

    run_vote_gov_proposal(&mut app, &contracts, USERS[0], id, VoteOutcome::Yes).unwrap();
    run_vote_gov_proposal(&mut app, &contracts, USERS[1], id, VoteOutcome::Yes).unwrap();

    increase_time_block(&mut app, VOTE_DURATION + 1);

    run_execute_proposal(&mut app, contracts.clone(), USERS[0], id).unwrap();
}
