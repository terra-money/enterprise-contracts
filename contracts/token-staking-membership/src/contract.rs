use common::cw::{Context, QueryContext};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
};
use cw2::set_contract_version;
use token_staking_api::error::TokenStakingResult;
use token_staking_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use token_staking_impl::execute::{claim, receive_cw20, unstake, update_config};
use token_staking_impl::query::{
    query_claims, query_config, query_releasable_claims, query_total_staked_amount,
    query_user_token_stake,
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

    match msg {
        ExecuteMsg::Unstake(msg) => unstake(ctx, msg),
        ExecuteMsg::Claim(msg) => claim(ctx, msg),
        ExecuteMsg::UpdateConfig(msg) => update_config(ctx, msg),
        ExecuteMsg::Receive(msg) => receive_cw20(ctx, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> TokenStakingResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> TokenStakingResult<Binary> {
    let qctx = QueryContext { deps, env };

    let response = match msg {
        QueryMsg::Config {} => to_binary(&query_config(&qctx)?)?,
        QueryMsg::UserStake(params) => to_binary(&query_user_token_stake(&qctx, params)?)?,
        QueryMsg::TotalStakedAmount(params) => {
            to_binary(&query_total_staked_amount(&qctx, params)?)?
        }
        QueryMsg::Claims(params) => to_binary(&query_claims(&qctx, params)?)?,
        QueryMsg::ReleasableClaims(params) => to_binary(&query_releasable_claims(&qctx, params)?)?,
    };

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> TokenStakingResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
