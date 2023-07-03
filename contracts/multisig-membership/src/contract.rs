use common::cw::{Context, QueryContext};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
};
use cw2::set_contract_version;
use membership_common::admin::update_admin;
use multisig_membership_api::error::MultisigMembershipResult;
use multisig_membership_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use multisig_membership_impl::execute::{set_members, update_members};
use multisig_membership_impl::query::{
    query_admin, query_members, query_total_weight, query_user_weight,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:multisig-membership";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> MultisigMembershipResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let ctx = &mut Context { deps, env, info };

    multisig_membership_impl::instantiate::instantiate(ctx, msg)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> MultisigMembershipResult<Response> {
    let ctx = &mut Context { deps, env, info };

    let response = match msg {
        ExecuteMsg::UpdateMembers(msg) => update_members(ctx, msg)?,
        ExecuteMsg::SetMembers(msg) => set_members(ctx, msg)?,
        ExecuteMsg::UpdateAdmin(msg) => update_admin(ctx, msg)?,
        // ExecuteMsg::AddWeightChangeHook(msg) => add_weight_change_hook(ctx, msg)?,
        // ExecuteMsg::RemoveWeightChangeHook(msg) => remove_weight_change_hook(ctx, msg)?,
    };

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> MultisigMembershipResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> MultisigMembershipResult<Binary> {
    let qctx = QueryContext { deps, env };

    let response = match msg {
        QueryMsg::Admin {} => to_binary(&query_admin(&qctx)?)?,
        QueryMsg::UserWeight(params) => to_binary(&query_user_weight(&qctx, params)?)?,
        QueryMsg::TotalWeight(params) => to_binary(&query_total_weight(&qctx, params)?)?,
        QueryMsg::Members(params) => to_binary(&query_members(&qctx, params)?)?,
    };

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> MultisigMembershipResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
