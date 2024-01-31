use crate::facade_helpers::TestFacade;
use crate::factory_helpers::{
    create_dao, default_dao_council, default_dao_metadata, default_gov_config, get_first_dao,
    import_cw721_membership, new_nft_membership,
};
use crate::helpers::{
    startup_with_versioning, ADMIN, CODE_ID_CW20, CODE_ID_CW721, NFT_TOKEN1, USER1,
};
use crate::traits::IntoAddr;
use crate::wasm_helpers::{assert_addr_code_id, assert_contract_admin};
use cw721::ContractInfoResponse;
use cw721_base::{Extension, MinterResponse};
use cw_multi_test::Executor;
use cw_utils::Duration;
use enterprise_facade_api::api::DaoType::Nft;
use enterprise_facade_common::facade::EnterpriseFacade;
use enterprise_factory_api::api::{CreateDaoMsg, NewCw721MembershipMsg};
use enterprise_governance_controller_api::api::GovConfig;

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
