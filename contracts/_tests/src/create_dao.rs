use crate::asset_helpers::cw20_unchecked;
use crate::cw20_helpers::Cw20Assert;
use crate::facade_helpers::{
    from_facade_dao_council, from_facade_gov_config, from_facade_metadata, TestFacade,
};
use crate::factory_helpers::{
    create_dao, default_create_dao_msg, default_dao_council, default_dao_metadata,
    default_gov_config, default_new_token_membership, default_token_marketing_info, get_first_dao,
    import_cw20_membership, import_cw3_membership, import_cw721_membership, new_denom_membership,
    new_multisig_membership, new_nft_membership, new_token_membership, query_all_daos,
};
use crate::helpers::{
    startup_with_versioning, ADDR_FACTORY, ADMIN, CODE_ID_ATTESTATION, CODE_ID_CW20, CODE_ID_CW3,
    CODE_ID_CW721, CODE_ID_ENTERPRISE, CODE_ID_FUNDS_DISTRIBUTOR, CODE_ID_GOVERNANCE,
    CODE_ID_GOV_CONTROLLER, CODE_ID_MEMBERSHIP_MULTISIG, CODE_ID_OUTPOSTS, CODE_ID_TREASURY,
    CW20_TOKEN1, CW20_TOKEN2, NFT_TOKEN1, NFT_TOKEN2, USER1, USER2, USER3,
};
use crate::traits::{ImplApp, IntoAddr, IntoStringVec};
use crate::wasm_helpers::{assert_addr_code_id, assert_contract_admin};
use attestation_api::api::AttestationTextResponse;
use attestation_api::msg::QueryMsg::AttestationText;
use cosmwasm_std::{coins, Decimal};
use cw20::{Cw20Coin, Cw20Contract, LogoInfo, MarketingInfoResponse};
use cw3_fixed_multisig::msg::Voter;
use cw721::ContractInfoResponse;
use cw721_base::{Extension, MinterResponse};
use cw_asset::{AssetInfo, AssetInfoUnchecked};
use cw_multi_test::Executor;
use cw_utils::{Duration, Threshold};
use enterprise_facade_api::api::DaoType;
use enterprise_facade_api::api::DaoType::Denom;
use enterprise_facade_common::facade::EnterpriseFacade;
use enterprise_factory_api::api::{
    CreateDaoMsg, DaoRecord, NewCw20MembershipMsg, NewCw721MembershipMsg, TokenMarketingInfo,
};
use enterprise_governance_controller_api::api::{DaoCouncilSpec, GovConfig, ProposalActionType};
use enterprise_protocol::api::{DaoMetadata, DaoSocialData, Logo};
use enterprise_versioning_api::api::Version;
use DaoType::{Multisig, Nft, Token};

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

    // verify that code IDs match the expected ones
    assert_addr_code_id(&app, &facade.enterprise_addr(), CODE_ID_ENTERPRISE);
    assert_addr_code_id(
        &app,
        &facade.funds_distributor_addr(),
        CODE_ID_FUNDS_DISTRIBUTOR,
    );
    assert_addr_code_id(&app, &facade.governance_addr(), CODE_ID_GOVERNANCE);
    assert_addr_code_id(&app, &facade.gov_controller_addr(), CODE_ID_GOV_CONTROLLER);
    assert_addr_code_id(&app, &facade.outposts_addr(), CODE_ID_OUTPOSTS);
    assert_addr_code_id(&app, &facade.treasury_addr(), CODE_ID_TREASURY);
    assert_addr_code_id(
        &app,
        &facade.council_membership_addr(),
        CODE_ID_MEMBERSHIP_MULTISIG,
    );
    assert_addr_code_id(&app, &facade.attestation_addr(), CODE_ID_ATTESTATION);

    assert_eq!(facade.factory_addr(), ADDR_FACTORY);

    // assert admins are correctly set for each of the contracts
    let enterprise_addr = facade.enterprise_addr().to_string();
    assert_contract_admin(&app, &facade.enterprise_addr(), &enterprise_addr);
    assert_contract_admin(&app, &facade.funds_distributor_addr(), &enterprise_addr);
    assert_contract_admin(&app, &facade.governance_addr(), &enterprise_addr);
    assert_contract_admin(&app, &facade.gov_controller_addr(), &enterprise_addr);
    assert_contract_admin(&app, &facade.outposts_addr(), &enterprise_addr);
    assert_contract_admin(&app, &facade.treasury_addr(), &enterprise_addr);
    assert_contract_admin(&app, &facade.membership_addr(), &enterprise_addr);
    assert_contract_admin(&app, &facade.council_membership_addr(), &enterprise_addr);
    // TODO: fix this
    // assert_contract_admin(
    //     &app,
    //     &facade.attestation_addr(),
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

    facade
        .council_membership()
        .assert_user_weights(vec![(USER1, 1), (USER2, 1), (USER3, 0)]);

    let attestation_text_resp: AttestationTextResponse = app
        .wrap()
        .query_wasm_smart(facade.attestation_addr().to_string(), &AttestationText {})?;
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

    facade
        .membership()
        .assert_user_weights(vec![(USER1, 1), (USER2, 2), (USER3, 5)]);
    facade.membership().assert_total_weight(8);

    facade.assert_dao_type(Multisig);

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

    let nft_config = facade.nft_config()?;

    assert_eq!(
        facade.query_dao_info()?.gov_config.unlocking_period,
        Duration::Time(300)
    );
    assert_eq!(nft_config.unlocking_period, Duration::Time(300));

    let dao_nft = nft_config.nft_contract;

    assert_contract_admin(&app, &dao_nft, facade.enterprise_addr().as_ref());
    assert_addr_code_id(&app, &dao_nft, CODE_ID_CW721);

    let nft_info: ContractInfoResponse = app
        .wrap()
        .query_wasm_smart(dao_nft.to_string(), &cw721::Cw721QueryMsg::ContractInfo {})?;

    assert_eq!(nft_membership.nft_name, nft_info.name);
    assert_eq!(nft_membership.nft_symbol, nft_info.symbol);

    let minter: MinterResponse = app.wrap().query_wasm_smart(
        dao_nft.to_string(),
        &cw721_base::msg::QueryMsg::<Extension>::Minter {},
    )?;
    assert_eq!(minter.minter, USER1.to_string());

    facade.assert_total_staked(0);

    let membership_contract = facade.membership();

    membership_contract.assert_total_weight(0);

    facade
        .funds_distributor()
        .assert_minimum_eligible_weight(2u8);

    // TODO: fix this
    // facade.assert_nft_whitelist(vec![NFT_TOKEN1, dao_nft.as_ref()]);

    facade.assert_dao_type(Nft);

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

    let nft_config = facade.nft_config()?;

    let dao_nft = nft_config.nft_contract;

    // verify that the minter is set to Enterprise contract
    let enterprise_contract = facade.enterprise_addr();
    let minter: MinterResponse = app.wrap().query_wasm_smart(
        dao_nft.to_string(),
        &cw721_base::msg::QueryMsg::<Extension>::Minter {},
    )?;
    assert_eq!(minter.minter, enterprise_contract.to_string());

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

    let nft_config = facade.nft_config()?;

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

    let membership_contract = facade.membership();

    membership_contract.assert_total_weight(0);

    facade
        .funds_distributor()
        .assert_minimum_eligible_weight(2u8);

    // TODO: fix this
    // facade.assert_nft_whitelist(vec![NFT_TOKEN1, dao_nft.as_ref()]);

    facade.assert_dao_type(Nft);

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

    let marketing_info = default_token_marketing_info();
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

    let token_config = facade.token_config()?;

    assert_eq!(
        facade.query_dao_info()?.gov_config.unlocking_period,
        Duration::Time(300)
    );
    assert_eq!(token_config.unlocking_period, Duration::Time(300));

    let dao_token = token_config.token_contract;

    let enterprise_contract = facade.enterprise_addr();
    assert_contract_admin(&app, &dao_token, enterprise_contract.as_ref());
    assert_addr_code_id(&app, &dao_token, CODE_ID_CW20);

    let dao_cw20_assert = Cw20Assert {
        app: &app,
        cw20_contract: &Cw20Contract(dao_token.clone()),
    };

    dao_cw20_assert.token_info(
        token_membership.token_name,
        token_membership.token_symbol,
        token_membership.token_decimals,
        user1_balance + user2_balance + dao_balance,
    );

    dao_cw20_assert.balances(vec![
        (USER1, user1_balance),
        (USER2, user2_balance),
        (USER3, 0),
        (facade.treasury_addr().as_ref(), dao_balance),
    ]);

    dao_cw20_assert.minter(USER3, Some(token_cap));

    facade.assert_total_staked(0);

    let membership_contract = facade.membership();

    membership_contract.assert_total_weight(0);

    facade
        .funds_distributor()
        .assert_minimum_eligible_weight(2u8);

    // TODO: fix this
    // facade.assert_asset_whitelist(vec![
    //     AssetInfo::cw20(CW20_TOKEN1.into_addr()),
    //     AssetInfo::cw20(dao_token),
    // ]);

    facade.assert_dao_type(Token);

    Ok(())
}

#[test]
fn create_new_token_dao_without_marketing_owner_sets_gov_controller_marketing_owner(
) -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let marketing_info = TokenMarketingInfo {
        marketing_owner: None,
        ..default_token_marketing_info()
    };
    let token_membership = NewCw20MembershipMsg {
        token_marketing: Some(marketing_info.clone()),
        ..default_new_token_membership()
    };
    let msg = CreateDaoMsg {
        dao_membership: new_token_membership(token_membership.clone()),
        ..default_create_dao_msg()
    };

    create_dao(&mut app, msg)?;

    let dao_addr = get_first_dao(&app)?;
    let facade = TestFacade {
        app: &app,
        dao_addr,
    };

    let dao_cw20_assert = Cw20Assert {
        app: &app,
        cw20_contract: &Cw20Contract(facade.token_config()?.token_contract),
    };

    // TODO: fix this, it's currently using enterprise address instead of governance controller
    dao_cw20_assert.marketing_info(MarketingInfoResponse {
        marketing: Some(facade.enterprise_addr()),
        project: marketing_info.project,
        description: marketing_info.description,
        logo: marketing_info.logo_url.map(LogoInfo::Url),
    });

    Ok(())
}

#[test]
fn create_new_token_dao_without_marketing_info_sets_gov_controller_marketing_owner(
) -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let token_membership = NewCw20MembershipMsg {
        token_marketing: None,
        ..default_new_token_membership()
    };
    let msg = CreateDaoMsg {
        dao_membership: new_token_membership(token_membership.clone()),
        ..default_create_dao_msg()
    };

    create_dao(&mut app, msg)?;

    let dao_addr = get_first_dao(&app)?;
    let facade = TestFacade {
        app: &app,
        dao_addr,
    };

    let dao_cw20_assert = Cw20Assert {
        app: &app,
        cw20_contract: &Cw20Contract(facade.token_config()?.token_contract),
    };

    // TODO: fix this, it's currently using enterprise address instead of governance controller
    dao_cw20_assert.marketing_info(MarketingInfoResponse {
        marketing: Some(facade.enterprise_addr()),
        project: None,
        description: None,
        logo: None,
    });

    Ok(())
}

#[test]
fn create_new_token_dao_without_minter_or_balances_fails() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let token_membership = NewCw20MembershipMsg {
        initial_dao_balance: None,
        initial_token_balances: vec![],
        token_mint: None,
        ..default_new_token_membership()
    };
    let msg = CreateDaoMsg {
        dao_membership: new_token_membership(token_membership.clone()),
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn create_new_token_dao_unlocking_shorter_than_voting_fails() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let token_membership = NewCw20MembershipMsg {
        unlocking_period: Duration::Time(100),
        ..default_new_token_membership()
    };
    let msg = CreateDaoMsg {
        dao_membership: new_token_membership(token_membership.clone()),
        gov_config: GovConfig {
            vote_duration: 101,
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn import_cw20_token_dao() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let user1_balance = 10u8;
    let cw20_contract = app
        .instantiate_contract(
            CODE_ID_CW20,
            ADMIN.into_addr(),
            &cw20_base::msg::InstantiateMsg {
                name: "Existing token".to_string(),
                symbol: "ETKN".to_string(),
                decimals: 8,
                initial_balances: vec![Cw20Coin {
                    address: USER1.to_string(),
                    amount: user1_balance.into(),
                }],
                mint: Some(cw20::MinterResponse {
                    minter: USER3.to_string(),
                    cap: None,
                }),
                marketing: None,
            },
            &[],
            "CW20 contract",
            Some(ADMIN.to_string()),
        )
        .unwrap();

    let msg = CreateDaoMsg {
        dao_metadata: default_dao_metadata(),
        gov_config: default_gov_config(),
        dao_council: Some(default_dao_council()),
        dao_membership: import_cw20_membership(cw20_contract.to_string(), 300),
        asset_whitelist: Some(vec![
            AssetInfoUnchecked::cw20(CW20_TOKEN1),
            AssetInfoUnchecked::cw20(CW20_TOKEN2),
        ]),
        nft_whitelist: None,
        minimum_weight_for_rewards: Some(3u8.into()),
        cross_chain_treasuries: None,
        attestation_text: None,
    };

    create_dao(&mut app, msg)?;

    let dao_addr = get_first_dao(&app)?;
    let facade = TestFacade {
        app: &app,
        dao_addr,
    };

    let token_config = facade.token_config()?;

    assert_eq!(token_config.token_contract, cw20_contract);

    assert_eq!(
        facade.query_dao_info()?.gov_config.unlocking_period,
        Duration::Time(300),
    );
    assert_eq!(token_config.unlocking_period, Duration::Time(300));

    let dao_cw20_assert = Cw20Assert {
        app: &app,
        cw20_contract: &Cw20Contract(cw20_contract.clone()),
    };

    dao_cw20_assert.token_info("Existing token", "ETKN", 8, user1_balance);

    dao_cw20_assert.balances(vec![(USER1, user1_balance)]);

    dao_cw20_assert.minter(USER3, None);

    facade.assert_total_staked(0);

    let membership_contract = facade.membership();

    membership_contract.assert_total_weight(0);

    facade
        .funds_distributor()
        .assert_minimum_eligible_weight(3u8);

    // TODO: fix this
    // facade.assert_asset_whitelist(vec![
    //     AssetInfo::cw20(CW20_TOKEN1.into_addr()),
    //     AssetInfo::cw20(CW20_TOKEN2.into_addr()),
    //     AssetInfo::cw20(cw20_contract),
    // ]);

    facade.assert_dao_type(Token);

    Ok(())
}

#[test]
fn import_cw20_token_dao_with_no_minter_or_supply_fails() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let cw20_contract = app
        .instantiate_contract(
            CODE_ID_CW20,
            ADMIN.into_addr(),
            &cw20_base::msg::InstantiateMsg {
                name: "Existing token".to_string(),
                symbol: "ETKN".to_string(),
                decimals: 8,
                initial_balances: vec![],
                mint: None,
                marketing: None,
            },
            &[],
            "CW20 contract",
            Some(ADMIN.to_string()),
        )
        .unwrap();

    let msg = CreateDaoMsg {
        dao_membership: import_cw20_membership(cw20_contract.to_string(), 300),
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn import_non_cw20_token_fails() -> anyhow::Result<()> {
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
        dao_membership: import_cw20_membership(cw721_contract.to_string(), 300),
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn import_cw20_token_dao_unlocking_period_less_than_voting_fails() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let cw20_contract = app
        .instantiate_contract(
            CODE_ID_CW20,
            ADMIN.into_addr(),
            &cw20_base::msg::InstantiateMsg {
                name: "Existing token".to_string(),
                symbol: "ETKN".to_string(),
                decimals: 8,
                initial_balances: vec![],
                mint: None,
                marketing: None,
            },
            &[],
            "CW20 contract",
            Some(ADMIN.to_string()),
        )
        .unwrap();

    let msg = CreateDaoMsg {
        dao_membership: import_cw20_membership(cw20_contract.to_string(), 100),
        gov_config: GovConfig {
            vote_duration: 101,
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn new_denom_dao() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let dao_denom = "dao_denom";

    let user1_balance = 10u8;

    app.mint_native(vec![(USER1, coins(user1_balance.into(), dao_denom))]);

    let msg = CreateDaoMsg {
        dao_metadata: default_dao_metadata(),
        gov_config: default_gov_config(),
        dao_council: Some(default_dao_council()),
        dao_membership: new_denom_membership(dao_denom, 300),
        asset_whitelist: Some(vec![AssetInfoUnchecked::cw20(CW20_TOKEN2)]),
        nft_whitelist: None,
        minimum_weight_for_rewards: Some(3u8.into()),
        cross_chain_treasuries: None,
        attestation_text: None,
    };

    create_dao(&mut app, msg)?;

    let dao_addr = get_first_dao(&app)?;
    let facade = TestFacade {
        app: &app,
        dao_addr,
    };

    let denom_config = facade.denom_config()?;

    assert_eq!(denom_config.denom, dao_denom.to_string());

    assert_eq!(
        facade.query_dao_info()?.gov_config.unlocking_period,
        Duration::Time(300),
    );
    assert_eq!(denom_config.unlocking_period, Duration::Time(300));

    facade.assert_total_staked(0);

    let membership_contract = facade.membership();

    membership_contract.assert_total_weight(0);

    facade
        .funds_distributor()
        .assert_minimum_eligible_weight(3u8);

    // TODO: fix this
    // facade.assert_asset_whitelist(vec![
    //     AssetInfo::cw20(CW20_TOKEN2.into_addr()),
    //     AssetInfo::native(dao_denom),
    // ]);

    facade.assert_dao_type(Denom);

    Ok(())
}

#[test]
fn new_denom_dao_unlocking_period_less_than_voting_fails() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_denom_membership("dao_denom", 100),
        gov_config: GovConfig {
            vote_duration: 101,
            ..default_gov_config()
        },
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn minimum_weight_for_rewards_set_to_none_defaults_to_zero() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        minimum_weight_for_rewards: None,
        ..default_create_dao_msg()
    };

    create_dao(&mut app, msg)?;

    let dao_addr = get_first_dao(&app)?;
    let facade = TestFacade {
        app: &app,
        dao_addr,
    };

    facade
        .funds_distributor()
        .assert_minimum_eligible_weight(0u8);

    Ok(())
}
