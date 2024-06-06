use crate::helpers::cw_multitest_helpers::USER1;
use crate::helpers::facade_helpers::facade;
use crate::traits::ImplApp;
use cosmwasm_std::{coins, Addr};
use cw_multi_test::{App, AppResponse, Executor};
use funds_distributor_api::api::{ClaimRewardsMsg, DistributionType};
use funds_distributor_api::msg::ExecuteMsg::{ClaimRewards, DistributeNative};

pub fn distribute_native_funds(
    app: &mut App,
    distributor: &str,
    denom: &str,
    amount: u128,
    distribution_type: DistributionType,
    dao: Addr,
) -> anyhow::Result<AppResponse> {
    app.mint_native(vec![(distributor, coins(amount, denom))]);
    app.execute_contract(
        Addr::unchecked(distributor),
        facade(&app, dao).funds_distributor().addr,
        &DistributeNative {
            distribution_type: Some(distribution_type),
        },
        &coins(amount, denom),
    )
}

pub fn claim_native_rewards(
    app: &mut App,
    user: &str,
    denom: &str,
    dao: Addr,
) -> anyhow::Result<AppResponse> {
    app.execute_contract(
        Addr::unchecked(user),
        facade(&app, dao).funds_distributor().addr,
        &ClaimRewards(ClaimRewardsMsg {
            user: USER1.to_string(),
            native_denoms: vec![denom.to_string()],
            cw20_assets: vec![],
        }),
        &vec![],
    )
}
