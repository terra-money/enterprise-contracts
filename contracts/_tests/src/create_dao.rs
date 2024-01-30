use crate::asset_helpers::cw20_unchecked;
use crate::facade_helpers::{
    from_facade_dao_council, from_facade_gov_config, from_facade_metadata, TestFacade,
};
use crate::factory_helpers::{
    create_dao, default_dao_council, default_dao_metadata, default_gov_config,
    default_new_token_membership, get_first_dao, import_cw3_membership, import_cw721_membership,
    new_multisig_membership, new_nft_membership, new_token_membership, query_all_daos,
};
use crate::helpers::{
    startup_with_versioning, ADDR_FACTORY, ADMIN, CODE_ID_ATTESTATION, CODE_ID_CW20, CODE_ID_CW3,
    CODE_ID_CW721, CODE_ID_ENTERPRISE, CODE_ID_FUNDS_DISTRIBUTOR, CODE_ID_GOVERNANCE,
    CODE_ID_GOV_CONTROLLER, CODE_ID_MEMBERSHIP_MULTISIG, CODE_ID_OUTPOSTS, CODE_ID_TREASURY,
    CW20_TOKEN1, CW20_TOKEN2, NFT_TOKEN1, NFT_TOKEN2, USER1, USER2, USER3,
};
use crate::membership_helpers::TestMembershipContract;
use crate::traits::{IntoAddr, IntoStringVec};
use crate::wasm_helpers::{assert_addr_code_id, assert_contract_admin};
use attestation_api::api::AttestationTextResponse;
use attestation_api::msg::QueryMsg::AttestationText;
use cosmwasm_std::{Decimal, Uint128};
use cw20::Cw20QueryMsg::Balance;
use cw20::{BalanceResponse, Cw20Coin, TokenInfoResponse};
use cw3_fixed_multisig::msg::Voter;
use cw721::ContractInfoResponse;
use cw721_base::{Extension, MinterResponse};
use cw_asset::{AssetInfo, AssetInfoUnchecked};
use cw_multi_test::Executor;
use cw_utils::{Duration, Threshold};
use enterprise_facade_api::api::DaoType;
use enterprise_facade_common::facade::EnterpriseFacade;
use enterprise_factory_api::api::{
    CreateDaoMsg, DaoRecord, NewCw20MembershipMsg, NewCw721MembershipMsg, TokenMarketingInfo,
};
use enterprise_governance_controller_api::api::{DaoCouncilSpec, GovConfig, ProposalActionType};
use enterprise_protocol::api::{DaoMetadata, DaoSocialData, Logo};
use enterprise_versioning_api::api::Version;
use funds_distributor_api::api::MinimumEligibleWeightResponse;
use funds_distributor_api::msg::QueryMsg::MinimumEligibleWeight;
use nft_staking_api::api::NftConfigResponse;
use nft_staking_api::msg::QueryMsg::NftConfig;
use token_staking_api::api::TokenConfigResponse;
use token_staking_api::msg::QueryMsg::TokenConfig;

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

    create_dao(&mut app, msg)?;

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

    // verify whitelists
    facade.assert_asset_whitelist(vec![
        AssetInfo::cw20(CW20_TOKEN1.into_addr()),
        AssetInfo::cw20(CW20_TOKEN2.into_addr()),
    ]);

    facade.assert_nft_whitelist(vec![NFT_TOKEN1, NFT_TOKEN2]);

    // verify council data
    let council = dao_info.dao_council.unwrap();
    assert_eq!(from_facade_dao_council(council), dao_council);
    // TODO: also verify this in council membership contract's weights?

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

    let msg = CreateDaoMsg {
        dao_metadata: default_dao_metadata(),
        gov_config: default_gov_config(),
        dao_council: Some(default_dao_council()),
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 2), (USER3, 5)]),
        asset_whitelist: None,
        nft_whitelist: None,
        minimum_weight_for_rewards: None,
        cross_chain_treasuries: None,
        attestation_text: None,
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

    let components = facade.query_component_contracts()?;
    let membership_contract = TestMembershipContract {
        app: &app,
        contract: components.membership_contract.unwrap(),
    };

    membership_contract.assert_user_weight(USER1, 1);
    membership_contract.assert_user_weight(USER2, 2);
    membership_contract.assert_user_weight(USER3, 5);

    membership_contract.assert_total_weight(8);

    assert_eq!(facade.query_dao_info()?.dao_type, DaoType::Multisig);

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
        cross_chain_treasuries: None,
        attestation_text: None,
    };

    create_dao(&mut app, msg)?;

    let dao_addr = get_first_dao(&app)?;
    let facade = TestFacade {
        app: &app,
        dao_addr,
    };

    assert_eq!(
        facade.member_info(USER1)?.voting_power,
        Decimal::from_ratio(1u8, 10u8)
    );

    assert_eq!(facade.member_info(USER2)?.voting_power, Decimal::zero());

    assert_eq!(
        facade.member_info(USER3)?.voting_power,
        Decimal::from_ratio(9u8, 10u8)
    );

    // TODO: this bunch of steps for verifying member weights in all places is copied over, extract it to a helper
    facade.assert_multisig_members_list(None, Some(1), vec![(USER1, 10)]);
    facade.assert_multisig_members_list(Some(USER1), None, vec![(USER3, 90)]);

    facade.assert_total_staked(0);

    let components = facade.query_component_contracts()?;
    let membership_contract = TestMembershipContract {
        app: &app,
        contract: components.membership_contract.unwrap(),
    };

    membership_contract.assert_user_weight(USER1, 10);
    membership_contract.assert_user_weight(USER2, 0);
    membership_contract.assert_user_weight(USER3, 90);

    membership_contract.assert_total_weight(100);

    // assert that we set different governance params from the ones in CW3, and ours are the ones used in the DAO
    assert_ne!(cw3_voting_time, gov_config.vote_duration);
    assert_ne!(cw3_threshold, gov_config.threshold);
    assert_eq!(
        from_facade_gov_config(facade.query_dao_info().unwrap().gov_config),
        gov_config
    );

    assert_eq!(facade.query_dao_info()?.dao_type, DaoType::Multisig);

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
        cross_chain_treasuries: None,
        attestation_text: None,
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn create_new_nft_dao() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let nft_membership = NewCw721MembershipMsg {
        nft_name: "DAO NFT".to_string(),
        nft_symbol: "DNFT".to_string(),
        minter: Some(USER1.to_string()),
        unlocking_period: Duration::Time(300),
    };
    let msg = CreateDaoMsg {
        dao_metadata: default_dao_metadata(),
        gov_config: default_gov_config(),
        dao_council: Some(default_dao_council()),
        dao_membership: new_nft_membership(nft_membership.clone()),
        asset_whitelist: None,
        nft_whitelist: Some(vec![NFT_TOKEN1.to_string()]),
        minimum_weight_for_rewards: Some(2u8.into()),
        cross_chain_treasuries: None,
        attestation_text: None,
    };

    create_dao(&mut app, msg)?;

    let dao_addr = get_first_dao(&app)?;
    let facade = TestFacade {
        app: &app,
        dao_addr,
    };

    // TODO: extract this tedious process to a helper (get to the NFT contract directly)
    let membership_contract = facade
        .query_component_contracts()?
        .membership_contract
        .unwrap();
    let nft_config: NftConfigResponse = app
        .wrap()
        .query_wasm_smart(membership_contract.to_string(), &NftConfig {})?;

    assert_eq!(
        facade.query_dao_info()?.gov_config.unlocking_period,
        Duration::Time(300)
    );
    assert_eq!(nft_config.unlocking_period, Duration::Time(300));

    let dao_nft = nft_config.nft_contract;

    let enterprise_contract = facade.query_component_contracts()?.enterprise_contract;
    let dao_nft_contract_info = app.wrap().query_wasm_contract_info(dao_nft.to_string())?;
    assert_eq!(
        dao_nft_contract_info.admin.unwrap(),
        enterprise_contract.to_string()
    );
    assert_eq!(dao_nft_contract_info.code_id, CODE_ID_CW721);

    let nft_info: ContractInfoResponse = app
        .wrap()
        .query_wasm_smart(dao_nft.to_string(), &cw721::Cw721QueryMsg::ContractInfo {})?;

    assert_eq!(nft_membership.nft_name, nft_info.name);
    assert_eq!(nft_membership.nft_symbol, nft_info.symbol);

    let minter: MinterResponse = app.wrap().query_wasm_smart(
        dao_nft.to_string(),
        &cw721_base::msg::QueryMsg::<Extension>::Minter {},
    )?;
    assert_eq!(minter.minter.unwrap(), USER1.to_string());

    facade.assert_total_staked(0);

    let components = facade.query_component_contracts()?;
    let membership_contract = TestMembershipContract {
        app: &app,
        contract: components.membership_contract.unwrap(),
    };

    membership_contract.assert_total_weight(0);

    let funds_distributor = facade
        .query_component_contracts()
        .unwrap()
        .funds_distributor_contract;
    let minimum_weight_for_rewards: MinimumEligibleWeightResponse = app
        .wrap()
        .query_wasm_smart(funds_distributor.to_string(), &MinimumEligibleWeight {})?;
    assert_eq!(
        minimum_weight_for_rewards.minimum_eligible_weight,
        Uint128::from(2u8)
    );

    // TODO: fix this
    // facade.assert_nft_whitelist(vec![NFT_TOKEN1, dao_nft.as_ref()]);

    assert_eq!(facade.query_dao_info()?.dao_type, DaoType::Nft);

    Ok(())
}

#[test]
fn create_new_nft_dao_without_minter() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_metadata: default_dao_metadata(),
        gov_config: default_gov_config(),
        dao_council: Some(default_dao_council()),
        dao_membership: new_nft_membership(NewCw721MembershipMsg {
            nft_name: "DAO NFT".to_string(),
            nft_symbol: "DNFT".to_string(),
            minter: None,
            unlocking_period: Duration::Time(300),
        }),
        asset_whitelist: None,
        nft_whitelist: None,
        minimum_weight_for_rewards: None,
        cross_chain_treasuries: None,
        attestation_text: None,
    };

    create_dao(&mut app, msg)?;

    let dao_addr = get_first_dao(&app)?;
    let facade = TestFacade {
        app: &app,
        dao_addr,
    };

    // TODO: extract this tedious process to a helper (get to the NFT contract directly)
    let membership_contract = facade
        .query_component_contracts()?
        .membership_contract
        .unwrap();
    let nft_config: NftConfigResponse = app
        .wrap()
        .query_wasm_smart(membership_contract.to_string(), &NftConfig {})?;

    let dao_nft = nft_config.nft_contract;

    // verify that the minter is set to Enterprise contract
    let enterprise_contract = facade.query_component_contracts()?.enterprise_contract;
    let minter: MinterResponse = app.wrap().query_wasm_smart(
        dao_nft.to_string(),
        &cw721_base::msg::QueryMsg::<Extension>::Minter {},
    )?;
    assert_eq!(minter.minter.unwrap(), enterprise_contract.to_string());

    Ok(())
}

#[test]
fn import_cw721_dao() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let cw721_contract = app
        .instantiate_contract(
            CODE_ID_CW721,
            ADMIN.into_addr(),
            &cw721_base::msg::InstantiateMsg {
                name: "Existing NFT".to_string(),
                symbol: "ENFT".to_string(),
                minter: USER1.to_string(),
            },
            &[],
            "CW721 contract",
            Some(ADMIN.to_string()),
        )
        .unwrap();

    let msg = CreateDaoMsg {
        dao_metadata: default_dao_metadata(),
        gov_config: default_gov_config(),
        dao_council: Some(default_dao_council()),
        dao_membership: import_cw721_membership(cw721_contract.to_string(), 300),
        asset_whitelist: None,
        nft_whitelist: Some(vec![NFT_TOKEN1.to_string()]),
        minimum_weight_for_rewards: Some(2u8.into()),
        cross_chain_treasuries: None,
        attestation_text: None,
    };

    create_dao(&mut app, msg)?;

    let dao_addr = get_first_dao(&app)?;
    let facade = TestFacade {
        app: &app,
        dao_addr,
    };

    // TODO: extract this tedious process to a helper (get to the NFT contract directly)
    let membership_contract = facade
        .query_component_contracts()?
        .membership_contract
        .unwrap();
    let nft_config: NftConfigResponse = app
        .wrap()
        .query_wasm_smart(membership_contract.to_string(), &NftConfig {})?;

    assert_eq!(
        facade.query_dao_info()?.gov_config.unlocking_period,
        Duration::Time(300)
    );
    assert_eq!(nft_config.unlocking_period, Duration::Time(300));

    assert_eq!(nft_config.nft_contract, cw721_contract);

    let nft_info: ContractInfoResponse = app.wrap().query_wasm_smart(
        cw721_contract.to_string(),
        &cw721::Cw721QueryMsg::ContractInfo {},
    )?;

    assert_eq!("Existing NFT", &nft_info.name);
    assert_eq!("ENFT", &nft_info.symbol);

    facade.assert_total_staked(0);

    let components = facade.query_component_contracts()?;
    let membership_contract = TestMembershipContract {
        app: &app,
        contract: components.membership_contract.unwrap(),
    };

    membership_contract.assert_total_weight(0);

    let funds_distributor = facade
        .query_component_contracts()
        .unwrap()
        .funds_distributor_contract;
    let minimum_weight_for_rewards: MinimumEligibleWeightResponse = app
        .wrap()
        .query_wasm_smart(funds_distributor.to_string(), &MinimumEligibleWeight {})?;
    assert_eq!(
        minimum_weight_for_rewards.minimum_eligible_weight,
        Uint128::from(2u8)
    );

    // TODO: fix this
    // facade.assert_nft_whitelist(vec![NFT_TOKEN1, dao_nft.as_ref()]);

    assert_eq!(facade.query_dao_info()?.dao_type, DaoType::Nft);

    Ok(())
}

#[test]
fn import_non_cw721_dao_fails() -> anyhow::Result<()> {
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
        dao_membership: import_cw721_membership(cw20_contract.to_string(), 300),
        asset_whitelist: None,
        nft_whitelist: None,
        minimum_weight_for_rewards: None,
        cross_chain_treasuries: None,
        attestation_text: None,
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn create_new_nft_unstaking_shorter_than_voting_fails() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_metadata: default_dao_metadata(),
        gov_config: GovConfig {
            vote_duration: 100,
            ..default_gov_config()
        },
        dao_council: Some(default_dao_council()),
        dao_membership: new_nft_membership(NewCw721MembershipMsg {
            nft_name: "NFT name".to_string(),
            nft_symbol: "NFT".to_string(),
            minter: None,
            unlocking_period: Duration::Time(99),
        }),
        asset_whitelist: None,
        nft_whitelist: None,
        minimum_weight_for_rewards: None,
        cross_chain_treasuries: None,
        attestation_text: None,
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn create_new_token_dao() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let user1_balance = 10u8;
    let user2_balance = 20u8;
    let dao_balance = 50u8;
    let token_cap = 1000u32;

    let marketing_info = TokenMarketingInfo {
        project: Some("Some project name".to_string()),
        description: Some("Project description".to_string()),
        marketing_owner: Some("marketing_owner".to_string()),
        logo_url: Some("logo_url".to_string()),
    };
    let token_membership = NewCw20MembershipMsg {
        token_name: "New DAO token".to_string(),
        token_symbol: "NDTKN".to_string(),
        token_decimals: 6,
        initial_token_balances: vec![
            Cw20Coin {
                address: USER1.to_string(),
                amount: user1_balance.into(),
            },
            Cw20Coin {
                address: USER2.to_string(),
                amount: user2_balance.into(),
            },
        ],
        initial_dao_balance: Some(dao_balance.into()),
        token_mint: Some(cw20::MinterResponse {
            minter: USER3.to_string(),
            cap: Some(token_cap.into()),
        }),
        token_marketing: Some(marketing_info),
        unlocking_period: Duration::Time(300),
    };
    let msg = CreateDaoMsg {
        dao_metadata: default_dao_metadata(),
        gov_config: default_gov_config(),
        dao_council: Some(default_dao_council()),
        dao_membership: new_token_membership(token_membership.clone()),
        asset_whitelist: Some(vec![AssetInfoUnchecked::cw20(CW20_TOKEN1)]),
        nft_whitelist: None,
        minimum_weight_for_rewards: Some(2u8.into()),
        cross_chain_treasuries: None,
        attestation_text: None,
    };

    create_dao(&mut app, msg)?;

    let dao_addr = get_first_dao(&app)?;
    let facade = TestFacade {
        app: &app,
        dao_addr,
    };

    // TODO: extract this tedious process to a helper (get to the token contract directly)
    let membership_contract = facade
        .query_component_contracts()?
        .membership_contract
        .unwrap();
    let token_config: TokenConfigResponse = app
        .wrap()
        .query_wasm_smart(membership_contract.to_string(), &TokenConfig {})?;

    assert_eq!(
        facade.query_dao_info()?.gov_config.unlocking_period,
        Duration::Time(300)
    );
    assert_eq!(token_config.unlocking_period, Duration::Time(300));

    let dao_token = token_config.token_contract;

    let enterprise_contract = facade.query_component_contracts()?.enterprise_contract;
    assert_contract_admin(&app, &dao_token, enterprise_contract.as_ref());
    assert_addr_code_id(&app, &dao_token, CODE_ID_CW20);

    let token_info: TokenInfoResponse = app
        .wrap()
        .query_wasm_smart(dao_token.to_string(), &cw20::Cw20QueryMsg::TokenInfo {})?;

    assert_eq!(token_membership.token_name, token_info.name);
    assert_eq!(token_membership.token_symbol, token_info.symbol);
    assert_eq!(token_membership.token_decimals, token_info.decimals);
    assert_eq!(
        Uint128::from(user1_balance + user2_balance + dao_balance),
        token_info.total_supply
    );

    // TODO: extract to a helper
    let user1_balance_resp: BalanceResponse = app.wrap().query_wasm_smart(
        dao_token.to_string(),
        &Balance {
            address: USER1.to_string(),
        },
    )?;
    assert_eq!(user1_balance_resp.balance, Uint128::from(user1_balance));

    let user2_balance_resp: BalanceResponse = app.wrap().query_wasm_smart(
        dao_token.to_string(),
        &Balance {
            address: USER2.to_string(),
        },
    )?;
    assert_eq!(user2_balance_resp.balance, Uint128::from(user2_balance));

    let user3_balance_resp: BalanceResponse = app.wrap().query_wasm_smart(
        dao_token.to_string(),
        &Balance {
            address: USER3.to_string(),
        },
    )?;
    assert_eq!(user3_balance_resp.balance, Uint128::zero());

    let treasury_balance_resp: BalanceResponse = app.wrap().query_wasm_smart(
        dao_token.to_string(),
        &Balance {
            address: facade
                .query_component_contracts()?
                .enterprise_treasury_contract
                .unwrap()
                .to_string(),
        },
    )?;
    assert_eq!(treasury_balance_resp.balance, Uint128::from(dao_balance));

    let minter: cw20::MinterResponse = app
        .wrap()
        .query_wasm_smart(dao_token.to_string(), &cw20::Cw20QueryMsg::Minter {})?;
    assert_eq!(minter.minter, USER3.to_string());
    assert_eq!(minter.cap, Some(Uint128::from(token_cap)));

    facade.assert_total_staked(0);

    let components = facade.query_component_contracts()?;
    let membership_contract = TestMembershipContract {
        app: &app,
        contract: components.membership_contract.unwrap(),
    };

    membership_contract.assert_total_weight(0);

    let funds_distributor = facade
        .query_component_contracts()
        .unwrap()
        .funds_distributor_contract;
    let minimum_weight_for_rewards: MinimumEligibleWeightResponse = app
        .wrap()
        .query_wasm_smart(funds_distributor.to_string(), &MinimumEligibleWeight {})?;
    assert_eq!(
        minimum_weight_for_rewards.minimum_eligible_weight,
        Uint128::from(2u8)
    );

    // TODO: fix this
    // facade.assert_asset_whitelist(vec![
    //     AssetInfo::cw20(CW20_TOKEN1.into_addr()),
    //     AssetInfo::cw20(dao_token),
    // ]);

    assert_eq!(facade.query_dao_info()?.dao_type, DaoType::Token);

    Ok(())
}
