use crate::state::{CROSS_CHAIN_PROXIES, CROSS_CHAIN_TREASURIES, ENTERPRISE_CONTRACT};
use crate::validate::enterprise_governance_controller_caller_only;
use common::cw::{Context, QueryContext};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Reply, Response,
    StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use enterprise_outposts_api::api::{
    AddCrossChainProxyMsg, AddCrossChainTreasuryMsg, CrossChainDeploymentsParams,
    CrossChainDeploymentsResponse, CrossChainTreasuriesParams, CrossChainTreasuriesResponse,
    CrossChainTreasury,
};
use enterprise_outposts_api::error::EnterpriseOutpostsError::{
    ProxyAlreadyExistsForChainId, TreasuryAlreadyExistsForChainId,
};
use enterprise_outposts_api::error::EnterpriseOutpostsResult;
use enterprise_outposts_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use enterprise_outposts_api::response::{
    execute_add_cross_chain_proxy_response, execute_add_cross_chain_treasury_response,
    instantiate_response,
};

pub const INSTANTIATE_ATTESTATION_REPLY_ID: u64 = 1;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise-outposts";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DEFAULT_QUERY_LIMIT: u8 = 50;
pub const MAX_QUERY_LIMIT: u8 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> EnterpriseOutpostsResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ENTERPRISE_CONTRACT.save(
        deps.storage,
        &deps.api.addr_validate(&msg.enterprise_contract)?,
    )?;

    Ok(instantiate_response())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> EnterpriseOutpostsResult<Response> {
    let ctx = &mut Context { deps, env, info };

    match msg {
        ExecuteMsg::AddCrossChainProxy(msg) => add_cross_chain_proxy(ctx, msg),
        ExecuteMsg::AddCrossChainTreasury(msg) => add_cross_chain_treasury(ctx, msg),
    }
}

fn add_cross_chain_proxy(
    ctx: &mut Context,
    msg: AddCrossChainProxyMsg,
) -> EnterpriseOutpostsResult<Response> {
    enterprise_governance_controller_caller_only(ctx)?;

    if CROSS_CHAIN_PROXIES.has(ctx.deps.storage, msg.chain_id.clone()) {
        Err(ProxyAlreadyExistsForChainId)
    } else {
        CROSS_CHAIN_PROXIES.save(ctx.deps.storage, msg.chain_id, &msg.proxy_addr)?;

        Ok(execute_add_cross_chain_proxy_response())
    }
}

fn add_cross_chain_treasury(
    ctx: &mut Context,
    msg: AddCrossChainTreasuryMsg,
) -> EnterpriseOutpostsResult<Response> {
    enterprise_governance_controller_caller_only(ctx)?;

    if CROSS_CHAIN_TREASURIES.has(ctx.deps.storage, msg.chain_id.clone()) {
        Err(TreasuryAlreadyExistsForChainId)
    } else {
        CROSS_CHAIN_TREASURIES.save(ctx.deps.storage, msg.chain_id, &msg.treasury_addr)?;

        Ok(execute_add_cross_chain_treasury_response())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_: DepsMut, _: Env, _: Reply) -> EnterpriseOutpostsResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> EnterpriseOutpostsResult<Binary> {
    let qctx = QueryContext::from(deps, env);

    let response = match msg {
        QueryMsg::CrossChainTreasuries(params) => {
            to_binary(&query_cross_chain_treasuries(qctx, params)?)?
        }
        QueryMsg::CrossChainDeployments(params) => {
            to_binary(&query_cross_chain_deployments(qctx, params)?)?
        }
    };
    Ok(response)
}

fn query_cross_chain_treasuries(
    qctx: QueryContext,
    params: CrossChainTreasuriesParams,
) -> EnterpriseOutpostsResult<CrossChainTreasuriesResponse> {
    let start_after = params.start_after.map(Bound::exclusive);
    let limit = params
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT as u32)
        .min(MAX_QUERY_LIMIT as u32);

    let treasuries = CROSS_CHAIN_TREASURIES
        .range(qctx.deps.storage, start_after, None, Order::Ascending)
        .take(limit as usize)
        .map(|res| {
            res.map(|(chain_id, treasury_addr)| CrossChainTreasury {
                chain_id,
                treasury_addr,
            })
        })
        .collect::<StdResult<Vec<CrossChainTreasury>>>()?;

    Ok(CrossChainTreasuriesResponse { treasuries })
}

fn query_cross_chain_deployments(
    qctx: QueryContext,
    params: CrossChainDeploymentsParams,
) -> EnterpriseOutpostsResult<CrossChainDeploymentsResponse> {
    let proxy_addr = CROSS_CHAIN_PROXIES.may_load(qctx.deps.storage, params.chain_id.clone())?;
    let treasury_addr =
        CROSS_CHAIN_TREASURIES.may_load(qctx.deps.storage, params.chain_id.clone())?;

    Ok(CrossChainDeploymentsResponse {
        chain_id: params.chain_id,
        proxy_addr,
        treasury_addr,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> EnterpriseOutpostsResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
