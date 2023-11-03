use crate::ibc_hooks::IcsProxyCallbackType::{InstantiateProxy, InstantiateTreasury};
use crate::ibc_hooks::{
    derive_intermediate_sender, ibc_hooks_msg_to_ics_proxy_contract, IcsProxyCallback,
    IcsProxyInstantiateMsg, ICS_PROXY_CALLBACKS, ICS_PROXY_CALLBACK_LAST_ID,
    TERRA_CHAIN_BECH32_PREFIX,
};
use crate::state::{CROSS_CHAIN_PROXIES, CROSS_CHAIN_TREASURIES, ENTERPRISE_CONTRACT};
use crate::validate::enterprise_governance_controller_caller_only;
use common::cw::{Context, QueryContext};
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::WasmMsg::Instantiate;
use cosmwasm_std::{
    entry_point, to_json_binary, wasm_execute, Addr, Binary, Deps, DepsMut, Env, MessageInfo,
    Order, Reply, Response, StdResult, SubMsg, SubMsgResponse, SubMsgResult,
};
use cw2::set_contract_version;
use cw_asset::AssetInfoUnchecked;
use cw_storage_plus::Bound;
use cw_utils::parse_reply_instantiate_data;
use enterprise_outposts_api::api::{
    CrossChainDeploymentsParams, CrossChainDeploymentsResponse, CrossChainMsgSpec,
    CrossChainTreasuriesParams, CrossChainTreasuriesResponse, CrossChainTreasury,
    DeployCrossChainTreasuryMsg, ExecuteCrossChainTreasuryMsg, ExecuteMsgReplyCallbackMsg,
};
use enterprise_outposts_api::error::EnterpriseOutpostsError::{
    NoCrossChainDeploymentForGivenChainId, ProxyAlreadyExistsForChainId,
    TreasuryAlreadyExistsForChainId, Unauthorized,
};
use enterprise_outposts_api::error::EnterpriseOutpostsResult;
use enterprise_outposts_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use enterprise_outposts_api::response::{
    execute_deploy_cross_chain_proxy_response, execute_deploy_cross_chain_treasury_response,
    execute_execute_cross_chain_treasury_response, execute_execute_msg_reply_callback_response,
    instantiate_response,
};
use enterprise_protocol::api::ComponentContractsResponse;
use enterprise_protocol::msg::QueryMsg::ComponentContracts;

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
        ExecuteMsg::DeployCrossChainTreasury(msg) => deploy_cross_chain_treasury(ctx, msg),
        ExecuteMsg::ExecuteCrossChainTreasury(msg) => execute_cross_chain_treasury(ctx, msg),
        ExecuteMsg::ExecuteMsgReplyCallback(msg) => execute_msg_reply_callback(ctx, msg),
    }
}

fn deploy_cross_chain_treasury(
    ctx: &mut Context,
    msg: DeployCrossChainTreasuryMsg,
) -> EnterpriseOutpostsResult<Response> {
    enterprise_governance_controller_caller_only(ctx)?;

    let qctx = QueryContext {
        deps: ctx.deps.as_ref(),
        env: ctx.env.clone(),
    };
    let deployments_response = query_cross_chain_deployments(
        qctx,
        CrossChainDeploymentsParams {
            chain_id: msg.cross_chain_msg_spec.chain_id.clone(),
        },
    )?;

    if deployments_response.treasury_addr.is_some() {
        return Err(TreasuryAlreadyExistsForChainId);
    }

    match deployments_response.proxy_addr {
        Some(proxy_contract) => {
            // there is already a proxy contract owned by this DAO,
            // so we just go ahead and instantiate the treasury

            let instantiate_treasury_msg = instantiate_remote_treasury(
                ctx.deps.branch(),
                ctx.env.clone(),
                msg.enterprise_treasury_code_id,
                proxy_contract,
                msg.asset_whitelist,
                msg.nft_whitelist,
                msg.cross_chain_msg_spec,
            )?;

            Ok(execute_deploy_cross_chain_treasury_response()
                .add_submessage(instantiate_treasury_msg))
        }
        None => {
            // there is no proxy contract owned by this DAO on the given chain,
            // so we go ahead and instantiate the proxy first

            // TODO: should we disallow multiple ongoing instantiations for the same chain?

            let callback_id = ICS_PROXY_CALLBACK_LAST_ID
                .may_load(ctx.deps.storage)?
                .unwrap_or_default()
                + 1;
            ICS_PROXY_CALLBACK_LAST_ID.save(ctx.deps.storage, &callback_id)?;

            ICS_PROXY_CALLBACKS.save(
                ctx.deps.storage,
                callback_id,
                &IcsProxyCallback {
                    cross_chain_msg_spec: msg.cross_chain_msg_spec.clone(),
                    proxy_addr: msg.chain_global_proxy.clone(),
                    callback_type: InstantiateProxy {
                        deploy_treasury_msg: Box::new(msg.clone()),
                    },
                },
            )?;

            // calculate what the address of this contract will look like on the other chain
            // via IBC-hooks
            let ibc_hooks_governance_controller_addr = derive_intermediate_sender(
                &msg.cross_chain_msg_spec.dest_ibc_channel,
                ctx.env.contract.address.as_ref(),
                &msg.cross_chain_msg_spec.chain_bech32_prefix,
            )?;

            let instantiate_proxy_msg = ibc_hooks_msg_to_ics_proxy_contract(
                &ctx.env,
                Wasm(Instantiate {
                    admin: None,
                    code_id: msg.ics_proxy_code_id,
                    msg: to_json_binary(&IcsProxyInstantiateMsg {
                        allow_cross_chain_msgs: true,
                        owner: Some(ibc_hooks_governance_controller_addr),
                        whitelist: None,
                        msgs: None,
                    })?,
                    funds: vec![],
                    label: "Proxy contract".to_string(),
                }),
                msg.chain_global_proxy,
                msg.cross_chain_msg_spec,
                Some(callback_id),
            )?;
            Ok(execute_deploy_cross_chain_proxy_response().add_submessage(instantiate_proxy_msg))
        }
    }
}

fn add_cross_chain_proxy(
    ctx: &mut Context,
    chain_id: String,
    proxy_addr: String,
) -> EnterpriseOutpostsResult<()> {
    if CROSS_CHAIN_PROXIES.has(ctx.deps.storage, chain_id.clone()) {
        Err(ProxyAlreadyExistsForChainId)
    } else {
        CROSS_CHAIN_PROXIES.save(ctx.deps.storage, chain_id, &proxy_addr)?;

        Ok(())
    }
}

fn add_cross_chain_treasury(
    ctx: &mut Context,
    chain_id: String,
    treasury_addr: String,
) -> EnterpriseOutpostsResult<()> {
    if CROSS_CHAIN_TREASURIES.has(ctx.deps.storage, chain_id.clone()) {
        Err(TreasuryAlreadyExistsForChainId)
    } else {
        CROSS_CHAIN_TREASURIES.save(ctx.deps.storage, chain_id, &treasury_addr)?;

        Ok(())
    }
}

fn instantiate_remote_treasury(
    deps: DepsMut,
    env: Env,
    enterprise_treasury_code_id: u64,
    proxy_contract: String,
    asset_whitelist: Option<Vec<AssetInfoUnchecked>>,
    nft_whitelist: Option<Vec<String>>,
    cross_chain_msg_spec: CrossChainMsgSpec,
) -> EnterpriseOutpostsResult<SubMsg> {
    let callback_id = ICS_PROXY_CALLBACK_LAST_ID
        .may_load(deps.storage)?
        .unwrap_or_default()
        + 1;
    ICS_PROXY_CALLBACK_LAST_ID.save(deps.storage, &callback_id)?;

    // TODO: should we disallow multiple ongoing instantiations for the same chain?

    ICS_PROXY_CALLBACKS.save(
        deps.storage,
        callback_id,
        &IcsProxyCallback {
            cross_chain_msg_spec: cross_chain_msg_spec.clone(),
            proxy_addr: proxy_contract.clone(),
            callback_type: InstantiateTreasury {
                cross_chain_msg_spec: cross_chain_msg_spec.clone(),
            },
        },
    )?;

    let instantiate_treasury_msg = ibc_hooks_msg_to_ics_proxy_contract(
        &env,
        Wasm(Instantiate {
            admin: Some(proxy_contract.clone()),
            code_id: enterprise_treasury_code_id,
            msg: to_json_binary(&enterprise_treasury_api::msg::InstantiateMsg {
                admin: proxy_contract.clone(),
                asset_whitelist,
                nft_whitelist,
            })?,
            funds: vec![],
            label: "Proxy treasury".to_string(),
        }),
        proxy_contract,
        cross_chain_msg_spec,
        Some(callback_id),
    )?;
    Ok(instantiate_treasury_msg)
}

fn execute_cross_chain_treasury(
    ctx: &mut Context,
    msg: ExecuteCrossChainTreasuryMsg,
) -> EnterpriseOutpostsResult<Response> {
    enterprise_governance_controller_caller_only(ctx)?;

    let qctx = QueryContext {
        deps: ctx.deps.as_ref(),
        env: ctx.env.clone(),
    };
    let response = query_cross_chain_deployments(
        qctx,
        CrossChainDeploymentsParams {
            chain_id: msg.treasury_target.cross_chain_msg_spec.chain_id.clone(),
        },
    )?;

    let proxy_addr = response
        .proxy_addr
        .ok_or(NoCrossChainDeploymentForGivenChainId)?;
    let treasury_addr = response
        .treasury_addr
        .ok_or(NoCrossChainDeploymentForGivenChainId)?;

    let execute_treasury_submsg = ibc_hooks_msg_to_ics_proxy_contract(
        &ctx.env,
        wasm_execute(treasury_addr, &msg.msg, vec![])?.into(),
        proxy_addr,
        msg.treasury_target.cross_chain_msg_spec,
        None,
    )?;

    Ok(execute_execute_cross_chain_treasury_response().add_submessage(execute_treasury_submsg))
}

pub fn execute_msg_reply_callback(
    ctx: &mut Context,
    msg: ExecuteMsgReplyCallbackMsg,
) -> EnterpriseOutpostsResult<Response> {
    let ics_proxy_callback = ICS_PROXY_CALLBACKS.may_load(ctx.deps.storage, msg.callback_id)?;

    match ics_proxy_callback {
        Some(ics_proxy_callback) => {
            let sender = ctx.info.sender.clone();

            // calculate what the IBC-hooks-derived address should be for the proxy
            // we're expecting the reply from
            let derived_proxy_addr = derive_intermediate_sender(
                &ics_proxy_callback.cross_chain_msg_spec.src_ibc_channel,
                &ics_proxy_callback.proxy_addr,
                TERRA_CHAIN_BECH32_PREFIX,
            )?;

            let ibc_hooks_proxy_addr = ctx.deps.api.addr_validate(&derived_proxy_addr)?;

            if sender != ibc_hooks_proxy_addr {
                return Err(Unauthorized);
            }

            ICS_PROXY_CALLBACKS.remove(ctx.deps.storage, msg.callback_id);

            let reply = Reply {
                id: msg.callback_id as u64,
                result: SubMsgResult::Ok(SubMsgResponse {
                    events: msg.events,
                    data: msg.data,
                }),
            };

            match ics_proxy_callback.callback_type {
                InstantiateProxy {
                    deploy_treasury_msg,
                } => handle_instantiate_proxy_reply_callback(
                    ctx,
                    ics_proxy_callback.cross_chain_msg_spec.chain_id,
                    *deploy_treasury_msg,
                    reply,
                ),
                InstantiateTreasury { .. } => handle_instantiate_treasury_reply_callback(
                    ctx,
                    ics_proxy_callback.cross_chain_msg_spec.chain_id,
                    reply,
                ),
            }
        }
        None => Err(Unauthorized),
    }
}

fn handle_instantiate_proxy_reply_callback(
    ctx: &mut Context,
    chain_id: String,
    deploy_treasury_msg: DeployCrossChainTreasuryMsg,
    reply: Reply,
) -> EnterpriseOutpostsResult<Response> {
    let proxy_addr = parse_reply_instantiate_data(reply)?.contract_address;

    add_cross_chain_proxy(ctx, chain_id, proxy_addr.clone())?;

    let instantiate_treasury_submsg = instantiate_remote_treasury(
        ctx.deps.branch(),
        ctx.env.clone(),
        deploy_treasury_msg.enterprise_treasury_code_id,
        proxy_addr,
        deploy_treasury_msg.asset_whitelist,
        deploy_treasury_msg.nft_whitelist,
        deploy_treasury_msg.cross_chain_msg_spec,
    )?;

    let dao_address = query_main_dao_addr(ctx.deps.as_ref())?;

    Ok(
        execute_execute_msg_reply_callback_response(dao_address.to_string())
            .add_submessage(instantiate_treasury_submsg),
    )
}

fn handle_instantiate_treasury_reply_callback(
    ctx: &mut Context,
    chain_id: String,
    reply: Reply,
) -> EnterpriseOutpostsResult<Response> {
    let treasury_addr = parse_reply_instantiate_data(reply)?.contract_address;

    add_cross_chain_treasury(ctx, chain_id, treasury_addr)?;

    let dao_address = query_main_dao_addr(ctx.deps.as_ref())?;

    Ok(execute_execute_msg_reply_callback_response(
        dao_address.to_string(),
    ))
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
            to_json_binary(&query_cross_chain_treasuries(qctx, params)?)?
        }
        QueryMsg::CrossChainDeployments(params) => {
            to_json_binary(&query_cross_chain_deployments(qctx, params)?)?
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

fn query_main_dao_addr(deps: Deps) -> EnterpriseOutpostsResult<Addr> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(deps.storage)?;

    let component_contracts: ComponentContractsResponse = deps
        .querier
        .query_wasm_smart(enterprise_contract.to_string(), &ComponentContracts {})?;

    Ok(component_contracts.enterprise_treasury_contract)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> EnterpriseOutpostsResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
