use crate::helpers::cw_multitest_helpers::{
    startup_with_versioning, CW20_TOKEN1, ULUNA, USER1, USER2,
};
use crate::helpers::factory_helpers::{
    create_dao, create_dao_and_get_facade, default_create_dao_msg, new_multisig_membership,
};
use cw_asset::AssetInfoUnchecked;
use enterprise_factory_api::api::CreateDaoMsg;

#[test]
fn instantiate_with_duplicate_initial_weights_fails() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER1, 2)]),
        ..default_create_dao_msg()
    };

    let result = create_dao(&mut app, msg);

    assert!(result.is_err());

    Ok(())
}

#[test]
fn instantiate_stores_minimum_eligible_weight() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        minimum_weight_for_rewards: Some(5u8.into()),
        ..default_create_dao_msg()
    };

    let facade = create_dao_and_get_facade(&mut app, msg)?;

    facade
        .funds_distributor()
        .assert_minimum_eligible_weight(5u8);

    Ok(())
}

#[test]
fn rewards_after_instantiation_are_0() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_multisig_membership(vec![(USER1, 1), (USER2, 2)]),
        asset_whitelist: Some(vec![
            AssetInfoUnchecked::native(ULUNA),
            AssetInfoUnchecked::cw20(CW20_TOKEN1),
        ]),
        ..default_create_dao_msg()
    };

    let facade = create_dao_and_get_facade(&mut app, msg)?;

    for user in [USER1, USER2] {
        facade
            .funds_distributor()
            .assert_native_user_rewards(user, vec![(ULUNA, 0)]);
        facade
            .funds_distributor()
            .assert_cw20_user_rewards(user, vec![(CW20_TOKEN1, 0)]);
    }

    Ok(())
}
