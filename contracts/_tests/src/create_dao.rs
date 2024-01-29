use crate::asset_helpers::cw20_unchecked;
use crate::facade_helpers::{from_facade_gov_config, from_facade_metadata, TestFacade};
use crate::factory_helpers::{
    default_new_token_membership, get_first_dao, new_multisig_membership, new_token_membership,
    query_all_daos,
};
use crate::helpers::{
    startup_with_versioning, ADDR_FACTORY, CODE_ID_ATTESTATION, CODE_ID_ENTERPRISE,
    CODE_ID_FUNDS_DISTRIBUTOR, CODE_ID_GOVERNANCE, CODE_ID_GOV_CONTROLLER,
    CODE_ID_MEMBERSHIP_MULTISIG, CODE_ID_OUTPOSTS, CODE_ID_TREASURY, CW20_TOKEN1, CW20_TOKEN2,
    NFT_TOKEN1, NFT_TOKEN2, USER1, USER2, USER3, USER_DAO_CREATOR,
};
use crate::membership_helpers::TestMembershipContract;
use crate::traits::{IntoAddr, IntoStringVec};
use crate::wasm_helpers::{assert_addr_code_id, assert_contract_admin};
use attestation_api::api::AttestationTextResponse;
use attestation_api::msg::QueryMsg::AttestationText;
use cosmwasm_std::{Addr, Decimal};
use cw_asset::AssetInfo;
use cw_multi_test::Executor;
use enterprise_facade_api::api::{AssetWhitelistParams, NftWhitelistParams};
use enterprise_facade_common::facade::EnterpriseFacade;
use enterprise_factory_api::api::{CreateDaoMsg, DaoRecord};
use enterprise_governance_controller_api::api::{DaoCouncilSpec, GovConfig, ProposalActionType};
use enterprise_protocol::api::{DaoMetadata, DaoSocialData, Logo};
use enterprise_versioning_api::api::Version;

#[test]
fn create_dao_initializes_common_dao_data_properly() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let dao_metadata = DaoMetadata {
        name: "DAO name".to_string(),
        description: Some("DAO's description text".to_string()),
        logo: Logo::Url("www.logourl.link".to_string()),
        socials: DaoSocialData {
            github_username: Some("github handle".to_string()),
            discord_username: Some("discord handle".to_string()),
            twitter_username: Some("twitter handle".to_string()),
            telegram_username: Some("telegram handle".to_string()),
        },
    };
    let gov_config = GovConfig {
        quorum: Decimal::percent(31),
        threshold: Decimal::percent(52),
        veto_threshold: Some(Decimal::percent(17)),
        vote_duration: 250,
        minimum_deposit: Some(4u8.into()),
        allow_early_proposal_execution: false,
    };
    let dao_council = DaoCouncilSpec {
        members: vec![USER1, USER2].into_string(),
        quorum: Decimal::percent(34),
        threshold: Decimal::percent(54),
        allowed_proposal_action_types: Some(vec![
            ProposalActionType::DeployCrossChainTreasury,
            ProposalActionType::RemoveAttestation,
        ]),
    };
    let attestation_text = "Attestation text for this DAO";
    let msg = CreateDaoMsg {
        dao_metadata: dao_metadata.clone(),
        gov_config: gov_config.clone(),
        dao_council: Some(dao_council.clone()),
        dao_membership: new_token_membership(default_new_token_membership()),
        asset_whitelist: Some(vec![
            cw20_unchecked(CW20_TOKEN1),
            cw20_unchecked(CW20_TOKEN2),
        ]),
        nft_whitelist: Some(vec![NFT_TOKEN1, NFT_TOKEN2].into_string()),
        minimum_weight_for_rewards: Some(6u8.into()),
        cross_chain_treasuries: None, // TODO: how do we test this?
        attestation_text: Some(attestation_text.into()),
    };

    app.execute_contract(
        USER_DAO_CREATOR.into_addr(),
        ADDR_FACTORY.into_addr(),
        &enterprise_factory_api::msg::ExecuteMsg::CreateDao(Box::new(msg)),
        &[],
    )
    .unwrap();

    let dao_addr = get_first_dao(&app)?;

    let all_daos = query_all_daos(&app)?;
    assert_eq!(
        all_daos.daos,
        vec![DaoRecord {
            dao_id: 1u8.into(),
            dao_address: dao_addr.clone(),
        }]
    );

    let facade = TestFacade {
        app: &app,
        dao_addr,
    };
    let components = facade.query_component_contracts()?;

    // verify that code IDs match the expected ones
    assert_addr_code_id(&app, &components.enterprise_contract, CODE_ID_ENTERPRISE);
    assert_addr_code_id(
        &app,
        &components.funds_distributor_contract,
        CODE_ID_FUNDS_DISTRIBUTOR,
    );
    assert_addr_code_id(
        &app,
        components.enterprise_governance_contract.as_ref().unwrap(),
        CODE_ID_GOVERNANCE,
    );
    assert_addr_code_id(
        &app,
        &components
            .enterprise_governance_controller_contract
            .as_ref()
            .unwrap(),
        CODE_ID_GOV_CONTROLLER,
    );
    assert_addr_code_id(
        &app,
        &components.enterprise_outposts_contract.as_ref().unwrap(),
        CODE_ID_OUTPOSTS,
    );
    assert_addr_code_id(
        &app,
        &components.enterprise_treasury_contract.as_ref().unwrap(),
        CODE_ID_TREASURY,
    );
    assert_addr_code_id(
        &app,
        &components.council_membership_contract.as_ref().unwrap(),
        CODE_ID_MEMBERSHIP_MULTISIG,
    );
    assert_addr_code_id(
        &app,
        &components.attestation_contract.as_ref().unwrap(),
        CODE_ID_ATTESTATION,
    );

    assert_eq!(components.enterprise_factory_contract, ADDR_FACTORY);

    // assert admins are correctly set for each of the contracts
    let enterprise_addr = components.enterprise_contract.to_string();
    assert_contract_admin(&app, &components.enterprise_contract, &enterprise_addr);
    assert_contract_admin(
        &app,
        &components.funds_distributor_contract,
        &enterprise_addr,
    );
    assert_contract_admin(
        &app,
        components.enterprise_governance_contract.as_ref().unwrap(),
        &enterprise_addr,
    );
    assert_contract_admin(
        &app,
        &components
            .enterprise_governance_controller_contract
            .as_ref()
            .unwrap(),
        &enterprise_addr,
    );
    assert_contract_admin(
        &app,
        &components.enterprise_outposts_contract.as_ref().unwrap(),
        &enterprise_addr,
    );
    assert_contract_admin(
        &app,
        &components.enterprise_treasury_contract.as_ref().unwrap(),
        &enterprise_addr,
    );
    assert_contract_admin(
        &app,
        &components.membership_contract.as_ref().unwrap(),
        &enterprise_addr,
    );
    assert_contract_admin(
        &app,
        &components.council_membership_contract.as_ref().unwrap(),
        &enterprise_addr,
    );
    // TODO: fix this
    // assert_contract_admin(
    //     &app,
    //     &components.attestation_contract.as_ref().unwrap(),
    //     &enterprise_addr,
    // );

    // verify DAO info is in place
    let dao_info = facade.query_dao_info()?;
    assert_eq!(
        dao_info.dao_version,
        Version {
            major: 0,
            minor: 1,
            patch: 0,
        }
    );
    assert_eq!(from_facade_metadata(dao_info.metadata), dao_metadata);
    assert_eq!(from_facade_gov_config(dao_info.gov_config), gov_config); // TODO: verify veto_threshold is properly converted to default value

    // verify asset whitelist
    // TODO: probably extract to a helper
    let asset_whitelist = facade
        .query_asset_whitelist(AssetWhitelistParams {
            start_after: None,
            limit: None,
        })?
        .assets;
    assert_eq!(
        asset_whitelist,
        vec![
            AssetInfo::cw20(CW20_TOKEN1.into_addr()),
            AssetInfo::cw20(CW20_TOKEN2.into_addr()),
        ]
    );

    // verify NFT whitelist
    let nft_whitelist = facade
        .query_nft_whitelist(NftWhitelistParams {
            start_after: None,
            limit: None,
        })?
        .nfts;
    assert_eq!(nft_whitelist, vec![NFT_TOKEN1, NFT_TOKEN2]);

    // verify council data
    // TODO: extract a helper or sth, prettify this :(
    let council = dao_info.dao_council.unwrap();
    assert_eq!(dao_council.quorum, council.quorum);
    assert_eq!(dao_council.threshold, council.threshold);
    assert_eq!(
        dao_council
            .members
            .into_iter()
            .map(|it| it.into_addr())
            .collect::<Vec<Addr>>(),
        council.members
    );
    assert_eq!(
        dao_council
            .allowed_proposal_action_types
            .unwrap_or_default(),
        council.allowed_proposal_action_types
    );

    let attestation_addr = components.attestation_contract.unwrap();
    let attestation_text_resp: AttestationTextResponse = app
        .wrap()
        .query_wasm_smart(attestation_addr.to_string(), &AttestationText {})?;
    assert_eq!(attestation_text_resp.text, attestation_text.to_string());

    Ok(())
}

#[test]
fn create_new_multisig_dao() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    // TODO: extract this metadata to some helpers or functions for default/anonymous values
    let dao_metadata = DaoMetadata {
        name: "DAO name".to_string(),
        description: None,
        logo: Logo::None,
        socials: DaoSocialData {
            github_username: None,
            discord_username: None,
            twitter_username: None,
            telegram_username: None,
        },
    };
    let gov_config = GovConfig {
        quorum: Decimal::percent(31),
        threshold: Decimal::percent(52),
        veto_threshold: Some(Decimal::percent(17)),
        vote_duration: 250,
        minimum_deposit: None,
        allow_early_proposal_execution: false,
    };
    let dao_council = DaoCouncilSpec {
        members: vec![USER1, USER2].into_string(),
        quorum: Decimal::percent(34),
        threshold: Decimal::percent(54),
        allowed_proposal_action_types: Some(vec![
            ProposalActionType::DeployCrossChainTreasury,
            ProposalActionType::RemoveAttestation,
        ]),
    };
    let msg = CreateDaoMsg {
        dao_metadata: dao_metadata.clone(),
        gov_config: gov_config.clone(),
        dao_council: Some(dao_council.clone()),
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 2), (USER3, 5)]),
        asset_whitelist: None,
        nft_whitelist: None,
        minimum_weight_for_rewards: None,
        cross_chain_treasuries: None,
        attestation_text: None,
    };

    app.execute_contract(
        USER_DAO_CREATOR.into_addr(),
        ADDR_FACTORY.into_addr(),
        &enterprise_factory_api::msg::ExecuteMsg::CreateDao(Box::new(msg)),
        &[],
    )
    .unwrap();

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

    facade.assert_multisig_members(None, Some(1), vec![(USER1, 1)]);
    facade.assert_multisig_members(Some(USER1), None, vec![(USER2, 2), (USER3, 5)]);

    facade.assert_total_staked(0);

    let components = facade.query_component_contracts()?;
    let membership_contract = TestMembershipContract {
        app: &app,
        contract: components.membership_contract.unwrap(),
    };

    membership_contract.assert_user_weight(USER1, 1);
    membership_contract.assert_user_weight(USER2, 2);
    membership_contract.assert_user_weight(USER3, 5);

    membership_contract.assert_total_weight(8);

    Ok(())
}
