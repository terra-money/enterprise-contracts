use crate::helpers::cw_multitest_helpers::{startup_with_versioning, CW20_TOKEN2, USER1};
use crate::helpers::facade_helpers::TestFacade;
use crate::helpers::factory_helpers::{
    create_dao, default_create_dao_msg, default_dao_council, default_dao_metadata,
    default_gov_config, get_first_dao, new_denom_membership,
};
use crate::traits::ImplApp;
use cosmwasm_std::coins;
use cw_asset::AssetInfoUnchecked;
use cw_utils::Duration;
use enterprise_facade_api::api::DaoType::Denom;
use enterprise_facade_common::facade::EnterpriseFacade;
use enterprise_factory_api::api::CreateDaoMsg;
use enterprise_governance_controller_api::api::GovConfig;

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
