use crate::helpers::cw20_helpers::Cw20Assert;
use crate::helpers::cw_multitest_helpers::{
    startup_with_versioning, ADMIN, CODE_ID_CW20, CODE_ID_CW721, CW20_TOKEN1, CW20_TOKEN2, USER1,
    USER2, USER3,
};
use crate::helpers::facade_helpers::TestFacade;
use crate::helpers::factory_helpers::{
    create_dao, default_create_dao_msg, default_dao_council, default_dao_metadata,
    default_gov_config, default_new_token_membership, default_token_marketing_info, get_first_dao,
    import_cw20_membership, new_token_membership,
};
use crate::helpers::wasm_helpers::{assert_addr_code_id, assert_contract_admin};
use crate::traits::IntoAddr;
use cw20::{Cw20Coin, Cw20Contract, LogoInfo, MarketingInfoResponse};
use cw_asset::AssetInfoUnchecked;
use cw_multi_test::Executor;
use cw_utils::Duration;
use enterprise_facade_api::api::DaoType::Token;
use enterprise_facade_common::facade::EnterpriseFacade;
use enterprise_factory_api::api::{CreateDaoMsg, NewCw20MembershipMsg, TokenMarketingInfo};
use enterprise_governance_controller_api::api::GovConfig;

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
