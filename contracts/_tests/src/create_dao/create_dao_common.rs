// use crate::helpers::asset_helpers::cw20_unchecked;
// use crate::helpers::cw_multitest_helpers::{
//     startup_with_versioning, ADDR_FACTORY, CODE_ID_ENTERPRISE, CODE_ID_FUNDS_DISTRIBUTOR,
//     CODE_ID_GOVERNANCE, CODE_ID_GOV_CONTROLLER, CODE_ID_MEMBERSHIP_MULTISIG, CODE_ID_OUTPOSTS,
//     CODE_ID_TREASURY, CW20_TOKEN1, CW20_TOKEN2, NFT_TOKEN1, NFT_TOKEN2, USER1, USER2, USER3,
// };
// use crate::helpers::facade_helpers::{
//     from_facade_dao_council, from_facade_gov_config, from_facade_metadata, TestFacade,
// };
// use crate::helpers::factory_helpers::{
//     create_dao, default_create_dao_msg, default_new_token_membership, get_first_dao,
//     new_token_membership, query_all_daos,
// };
// use crate::helpers::wasm_helpers::{assert_addr_code_id, assert_contract_admin};
// use crate::traits::{IntoAddr, IntoStringVec};
// use cosmwasm_std::Decimal;
// use cw_asset::AssetInfo;
// use enterprise_facade_common::facade::EnterpriseFacade;
// use enterprise_factory_api::api::{CreateDaoMsg, DaoRecord};
// use enterprise_governance_controller_api::api::{DaoCouncilSpec, GovConfig, ProposalActionType};
// use enterprise_protocol::api::{DaoMetadata, DaoSocialData, Logo};
// use enterprise_versioning_api::api::Version;
// use std::cell::RefCell;
// use std::ops::{Deref, DerefMut};
//
// // TODO: can we parameterize those so that they test for each DAO type?
//
// #[test]
// fn create_dao_initializes_common_dao_data_properly() -> anyhow::Result<()> {
//     let mut app = startup_with_versioning();
//
//     let dao_metadata = DaoMetadata {
//         name: "DAO name".to_string(),
//         description: Some("DAO's description text".to_string()),
//         logo: Logo::Url("www.logo.link".to_string()),
//         socials: DaoSocialData {
//             github_username: Some("github handle".to_string()),
//             discord_username: Some("discord handle".to_string()),
//             twitter_username: Some("twitter handle".to_string()),
//             telegram_username: Some("telegram handle".to_string()),
//         },
//     };
//     let gov_config = GovConfig {
//         quorum: Decimal::percent(31),
//         threshold: Decimal::percent(52),
//         veto_threshold: Some(Decimal::percent(17)),
//         vote_duration: 250,
//         minimum_deposit: Some(4u8.into()),
//         allow_early_proposal_execution: false,
//     };
//     let dao_council = DaoCouncilSpec {
//         members: vec![USER1, USER2].into_string(),
//         quorum: Decimal::percent(34),
//         threshold: Decimal::percent(54),
//         allowed_proposal_action_types: Some(vec![ProposalActionType::DeployCrossChainTreasury]),
//     };
//     let msg = CreateDaoMsg {
//         dao_metadata: dao_metadata.clone(),
//         gov_config: gov_config.clone(),
//         dao_council: Some(dao_council.clone()),
//         dao_membership: new_token_membership(default_new_token_membership()),
//         asset_whitelist: Some(vec![
//             cw20_unchecked(CW20_TOKEN1),
//             cw20_unchecked(CW20_TOKEN2),
//         ]),
//         nft_whitelist: Some(vec![NFT_TOKEN1, NFT_TOKEN2].into_string()),
//         minimum_weight_for_rewards: Some(6u8.into()),
//         proposals_tracked_for_participation_rewards: None,
//         cross_chain_treasuries: None, // TODO: how do we test this?
//     };
//
//     let rc = RefCell::new(app);
//
//     create_dao(rc.borrow_mut().deref_mut(), msg)?;
//
//     let dao_addr = get_first_dao(rc.borrow().deref())?;
//
//     let all_daos = query_all_daos(rc.borrow().deref())?;
//     assert_eq!(
//         all_daos.daos,
//         vec![DaoRecord {
//             dao_id: 1u8.into(),
//             dao_address: dao_addr.clone(),
//         }]
//     );
//
//     let binding = rc.borrow();
//     // let facade = TestFacadeRc { app: &rc, dao_addr };
//
//     // verify that code IDs match the expected ones
//     // assert_addr_code_id(&app, &facade.enterprise_addr(), CODE_ID_ENTERPRISE);
//     // assert_addr_code_id(
//     //     &app,
//     //     &facade.funds_distributor_addr(),
//     //     CODE_ID_FUNDS_DISTRIBUTOR,
//     // );
//     assert_addr_code_id(
//         rc.borrow().deref(),
//         &facade.governance_addr(),
//         CODE_ID_GOVERNANCE,
//     );
//     // assert_addr_code_id(&app, &facade.gov_controller_addr(), CODE_ID_GOV_CONTROLLER);
//     // assert_addr_code_id(&app, &facade.outposts_addr(), CODE_ID_OUTPOSTS);
//     // assert_addr_code_id(&app, &facade.treasury_addr(), CODE_ID_TREASURY);
//     // assert_addr_code_id(
//     //     &app,
//     //     &facade.council_membership_addr(),
//     //     CODE_ID_MEMBERSHIP_MULTISIG,
//     // );
//     //
//     // assert_eq!(facade.factory_addr(), ADDR_FACTORY);
//     //
//     // // assert admins are correctly set for each of the contracts
//     // let enterprise_addr = facade.enterprise_addr().to_string();
//     // assert_contract_admin(&app, &facade.enterprise_addr(), &enterprise_addr);
//     // assert_contract_admin(&app, &facade.funds_distributor_addr(), &enterprise_addr);
//     // assert_contract_admin(&app, &facade.governance_addr(), &enterprise_addr);
//     // assert_contract_admin(&app, &facade.gov_controller_addr(), &enterprise_addr);
//     // assert_contract_admin(&app, &facade.outposts_addr(), &enterprise_addr);
//     // assert_contract_admin(&app, &facade.treasury_addr(), &enterprise_addr);
//     // assert_contract_admin(&app, &facade.membership_addr(), &enterprise_addr);
//     // assert_contract_admin(&app, &facade.council_membership_addr(), &enterprise_addr);
//     // TODO: fix this
//     // assert_contract_admin(
//     //     &app,
//     //     &facade.attestation_addr(),
//     //     &enterprise_addr,
//     // );
//
//     // verify DAO info is in place
//     let dao_info = facade.query_dao_info()?;
//     assert_eq!(
//         dao_info.dao_version,
//         Version {
//             major: 0,
//             minor: 1,
//             patch: 0,
//         }
//     );
//     assert_eq!(from_facade_metadata(dao_info.metadata), dao_metadata);
//     assert_eq!(from_facade_gov_config(dao_info.gov_config), gov_config); // TODO: verify veto_threshold is properly converted to default value
//
//     // verify whitelists
//     facade.assert_asset_whitelist(vec![
//         AssetInfo::cw20(CW20_TOKEN1.into_addr()),
//         AssetInfo::cw20(CW20_TOKEN2.into_addr()),
//     ]);
//
//     facade.assert_nft_whitelist(vec![NFT_TOKEN1, NFT_TOKEN2]);
//
//     // verify council data
//     let council = dao_info.dao_council.unwrap();
//     assert_eq!(from_facade_dao_council(council), dao_council);
//
//     facade
//         .council_membership()
//         .assert_user_weights(vec![(USER1, 1), (USER2, 1), (USER3, 0)]);
//
//     Ok(())
// }
//
// #[test]
// fn minimum_weight_for_rewards_set_to_none_defaults_to_zero() -> anyhow::Result<()> {
//     let mut app = startup_with_versioning();
//
//     let msg = CreateDaoMsg {
//         minimum_weight_for_rewards: None,
//         ..default_create_dao_msg()
//     };
//
//     create_dao(&mut app, msg)?;
//
//     let dao_addr = get_first_dao(&app)?;
//     let facade = TestFacade {
//         app: &app,
//         dao_addr,
//     };
//
//     facade
//         .funds_distributor()
//         .assert_minimum_eligible_weight(0u8);
//
//     Ok(())
// }
