use crate::claim::claim_rewards;
use crate::distributing::{distribute_cw20, distribute_native};
use crate::rewards::query_user_rewards;
use crate::state::{ENTERPRISE_CONTRACT, TOTAL_WEIGHT};
use crate::user_weights::update_user_weights;
use common::cw::{Context, QueryContext};
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, Uint128,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use funds_distributor_api::error::{DistributorError, DistributorResult};
use funds_distributor_api::msg::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:funds-distributor";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> DistributorResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let enterprise_contract = deps.api.addr_validate(&msg.enterprise_contract)?;
    ENTERPRISE_CONTRACT.save(deps.storage, &enterprise_contract)?;

    TOTAL_WEIGHT.save(deps.storage, &Uint128::zero())?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> DistributorResult<Response> {
    let ctx = &mut Context { deps, env, info };
    match msg {
        ExecuteMsg::UpdateUserWeights(msg) => update_user_weights(ctx, msg),
        ExecuteMsg::DistributeNative {} => distribute_native(ctx),
        ExecuteMsg::ClaimRewards(msg) => claim_rewards(ctx, msg),
        ExecuteMsg::Receive(msg) => receive_cw20(ctx, msg),
    }
}

fn receive_cw20(ctx: &mut Context, cw20_msg: Cw20ReceiveMsg) -> DistributorResult<Response> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Distribute {}) => distribute_cw20(ctx, cw20_msg),
        _ => Err(DistributorError::Std(StdError::generic_err(
            "msg payload not recognized",
        ))),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> DistributorResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> DistributorResult<Binary> {
    let qctx = QueryContext { deps, env };

    let response = match msg {
        QueryMsg::UserRewards(params) => to_binary(&query_user_rewards(qctx, params)?)?,
    };
    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> DistributorResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
