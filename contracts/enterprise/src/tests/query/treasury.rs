use cosmwasm_std::{Addr, Uint128};
use common::cw::testing::{mock_env, mock_info, mock_query_ctx};
use enterprise_protocol::api::ListMultisigMembersMsg;
use enterprise_protocol::error::DaoError::UnsupportedOperationForDaoType;
use enterprise_protocol::error::DaoResult;
use crate::contract::{query_cw20_treasury, query_list_multisig_members};
use crate::tests::helpers::{CW20_ADDR, existing_token_dao_membership, instantiate_stub_dao, stub_token_info};
use crate::tests::querier::mock_querier::mock_dependencies;

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
    )?;

    let treasury = query_cw20_treasury(mock_query_ctx(deps.as_ref(), &env))?;

    assert!(
        treasury.assets.is_empty(),
    );

    Ok(())
}