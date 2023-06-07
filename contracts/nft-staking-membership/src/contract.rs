use common::cw::{Context, QueryContext};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
};
use cw2::set_contract_version;
use nft_staking_api::error::NftStakingResult;
use nft_staking_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use nft_staking_impl::execute::{claim, receive_nft, unstake, update_config};
use nft_staking_impl::query::{
    query_claims, query_config, query_releasable_claims, query_stakers, query_total_staked_amount,
    query_user_nft_stake, query_user_total_stake,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:nft-staking-membership";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> NftStakingResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let ctx = &mut Context { deps, env, info };

    nft_staking_impl::instantiate::instantiate(ctx, msg)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> NftStakingResult<Response> {
    let ctx = &mut Context { deps, env, info };

    match msg {
        ExecuteMsg::Unstake(msg) => unstake(ctx, msg),
        ExecuteMsg::Claim(msg) => claim(ctx, msg),
        ExecuteMsg::UpdateConfig(msg) => update_config(ctx, msg),
        ExecuteMsg::ReceiveNft(msg) => receive_nft(ctx, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> NftStakingResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> NftStakingResult<Binary> {
    let qctx = QueryContext { deps, env };

    let response = match msg {
        QueryMsg::Config {} => to_binary(&query_config(&qctx)?)?,
        QueryMsg::UserStake(params) => to_binary(&query_user_nft_stake(&qctx, params)?)?,
        QueryMsg::UserTotalStake(params) => to_binary(&query_user_total_stake(&qctx, params)?)?,
        QueryMsg::TotalStakedAmount(params) => {
            to_binary(&query_total_staked_amount(&qctx, params)?)?
        }
        QueryMsg::Claims(params) => to_binary(&query_claims(&qctx, params)?)?,
        QueryMsg::ReleasableClaims(params) => to_binary(&query_releasable_claims(&qctx, params)?)?,
        QueryMsg::Stakers(params) => to_binary(&query_stakers(&qctx, params)?)?,
    };

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> NftStakingResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
