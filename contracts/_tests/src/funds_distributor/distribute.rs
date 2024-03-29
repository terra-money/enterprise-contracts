use enterprise_factory_api::api::CreateDaoMsg;
use crate::helpers::cw_multitest_helpers::{CW20_TOKEN1, startup_with_versioning, ULUNA};
use crate::helpers::facade_helpers::{facade};
use crate::helpers::factory_helpers::{asset_whitelist, create_dao_and_get_addr, default_create_dao_msg, default_new_token_membership, new_token_membership};

#[test]
fn distribute_total_weight_0_fails() -> anyhow::Result<()> {
    let mut app = startup_with_versioning();

    let msg = CreateDaoMsg {
        dao_membership: new_token_membership(default_new_token_membership()),
        asset_whitelist: asset_whitelist(vec![ULUNA], vec![CW20_TOKEN1]),
        ..default_create_dao_msg()
    };

    // let facade = create_dao_and_get_facade(&mut app, msg)?;

    let dao = create_dao_and_get_addr(&mut app, msg)?;

    // let result = facade(&app, dao.clone()).funds_distributor().distribute_native(&mut app, ULUNA, 1);
    //
    // assert!(result.is_err());

    Ok(())
}