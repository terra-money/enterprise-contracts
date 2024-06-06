use common::cw::{Context, QueryContext};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
};
use cw2::set_contract_version;
use denom_staking_api::error::DenomStakingResult;
use denom_staking_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use denom_staking_impl::execute::{claim, stake_denom, unstake, update_unlocking_period};
use denom_staking_impl::query::{
    query_claims, query_denom_config, query_members, query_releasable_claims, query_total_weight,
    query_user_weight,
};
use membership_common::member_weights::query_total_weight_above;
use membership_common::weight_change_hooks::{add_weight_change_hook, remove_weight_change_hook};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:denom-staking-membership";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> DenomStakingResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let ctx = &mut Context { deps, env, info };

    denom_staking_impl::instantiate::instantiate(ctx, msg)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> DenomStakingResult<Response> {
    let ctx = &mut Context { deps, env, info };

    let response = match msg {
        ExecuteMsg::Unstake(msg) => unstake(ctx, msg)?,
        ExecuteMsg::Claim(msg) => claim(ctx, msg)?,
        ExecuteMsg::UpdateUnlockingPeriod(msg) => update_unlocking_period(ctx, msg)?,
        ExecuteMsg::AddWeightChangeHook(msg) => add_weight_change_hook(ctx, msg)?,
        ExecuteMsg::RemoveWeightChangeHook(msg) => remove_weight_change_hook(ctx, msg)?,
        ExecuteMsg::Stake { user } => stake_denom(ctx, user)?,
    };

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> DenomStakingResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> DenomStakingResult<Binary> {
    let qctx = QueryContext { deps, env };

    let response = match msg {
        QueryMsg::DenomConfig {} => to_json_binary(&query_denom_config(&qctx)?)?,
        QueryMsg::UserWeight(params) => to_json_binary(&query_user_weight(&qctx, params)?)?,
        QueryMsg::TotalWeight(params) => to_json_binary(&query_total_weight(&qctx, params)?)?,
        QueryMsg::TotalWeightAbove(params) => {
            to_json_binary(&query_total_weight_above(&qctx, params)?)?
        }
        QueryMsg::Claims(params) => to_json_binary(&query_claims(&qctx, params)?)?,
        QueryMsg::ReleasableClaims(params) => {
            to_json_binary(&query_releasable_claims(&qctx, params)?)?
        }
        QueryMsg::Members(params) => to_json_binary(&query_members(&qctx, params)?)?,
    };

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> DenomStakingResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
