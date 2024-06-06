use common::cw::{Context, QueryContext};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
};
use cw2::set_contract_version;
use membership_common::member_weights::query_total_weight_above;
use membership_common::weight_change_hooks::{add_weight_change_hook, remove_weight_change_hook};
use token_staking_api::error::TokenStakingResult;
use token_staking_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use token_staking_impl::execute::{claim, receive_cw20, unstake, update_unlocking_period};
use token_staking_impl::migrate::migrate_to_v1_1_1;
use token_staking_impl::query::{
    query_claims, query_members, query_releasable_claims, query_token_config, query_total_weight,
    query_user_weight,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:token-staking-membership";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> TokenStakingResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let ctx = &mut Context { deps, env, info };

    token_staking_impl::instantiate::instantiate(ctx, msg)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> TokenStakingResult<Response> {
    let ctx = &mut Context { deps, env, info };

    let response = match msg {
        ExecuteMsg::Unstake(msg) => unstake(ctx, msg)?,
        ExecuteMsg::Claim(msg) => claim(ctx, msg)?,
        ExecuteMsg::UpdateUnlockingPeriod(msg) => update_unlocking_period(ctx, msg)?,
        ExecuteMsg::Receive(msg) => receive_cw20(ctx, msg)?,
        ExecuteMsg::AddWeightChangeHook(msg) => add_weight_change_hook(ctx, msg)?,
        ExecuteMsg::RemoveWeightChangeHook(msg) => remove_weight_change_hook(ctx, msg)?,
    };

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> TokenStakingResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> TokenStakingResult<Binary> {
    let qctx = QueryContext { deps, env };

    let response = match msg {
        QueryMsg::TokenConfig {} => to_json_binary(&query_token_config(&qctx)?)?,
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
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> TokenStakingResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let submsgs = migrate_to_v1_1_1(deps, env, msg)?;

    Ok(Response::new()
        .add_attribute("action", "migrate")
        .add_submessages(submsgs))
}
