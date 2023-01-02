use crate::contract::query_cw20_treasury;
use crate::tests::helpers::{
    existing_token_dao_membership, instantiate_stub_dao, stub_token_info, CW20_ADDR,
};
use crate::tests::querier::mock_querier::mock_dependencies;
use common::cw::testing::{mock_env, mock_info, mock_query_ctx};
use cosmwasm_std::{Addr, Uint128};
use enterprise_protocol::error::DaoResult;

// TODO: un-ignore once proper mock querier functions are added
#[ignore]
#[test]
fn cw20_treasury_lists_luna_and_dao_token_by_default() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked("dao_addr");
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    deps.querier
        .with_token_balances(&[(CW20_ADDR, &[("dao_addr", Uint128::zero())])]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
        None,
    )?;

    let treasury = query_cw20_treasury(mock_query_ctx(deps.as_ref(), &env))?;

    assert!(treasury.assets.is_empty(),);

    Ok(())
}
