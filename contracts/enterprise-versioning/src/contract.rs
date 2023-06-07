use crate::state::{ADMIN, VERSIONS};
use common::cw::{Context, QueryContext};
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use enterprise_versioning_api::api::{
    AddVersionMsg, AdminResponse, Version, VersionParams, VersionResponse, VersionsParams,
    VersionsResponse,
};
use enterprise_versioning_api::error::EnterpriseVersioningError::{
    Unauthorized, VersionAlreadyExists, VersionNotFound,
};
use enterprise_versioning_api::error::EnterpriseVersioningResult;
use enterprise_versioning_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise-versioning";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_QUERY_LIMIT: u8 = 10;
const MAX_QUERY_LIMIT: u8 = 50;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> EnterpriseVersioningResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = deps.api.addr_validate(&msg.admin)?;
    ADMIN.save(deps.storage, &admin)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", admin.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> EnterpriseVersioningResult<Response> {
    let ctx = &mut Context { deps, env, info };

    match msg {
        ExecuteMsg::AddVersion(msg) => add_version(ctx, msg),
    }
}

fn add_version(ctx: &mut Context, msg: AddVersionMsg) -> EnterpriseVersioningResult<Response> {
    let admin = ADMIN.load(ctx.deps.storage)?;

    if admin != ctx.info.sender {
        return Err(Unauthorized);
    }

    if VERSIONS.has(ctx.deps.storage, msg.version.version) {
        return Err(VersionAlreadyExists {
            version: msg.version.version,
        });
    }

    VERSIONS.save(ctx.deps.storage, msg.version.version, &msg.version)?;

    Ok(Response::new()
        .add_attribute("action", "add_version")
        .add_attribute("version", msg.version.version.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> EnterpriseVersioningResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> EnterpriseVersioningResult<Binary> {
    let qctx = QueryContext { deps, env };

    let response = match msg {
        QueryMsg::Admin {} => to_binary(&query_admin(&qctx)?)?,
        QueryMsg::Version(params) => to_binary(&query_version(&qctx, params)?)?,
        QueryMsg::Versions(params) => to_binary(&query_versions(&qctx, params)?)?,
    };

    Ok(response)
}

pub fn query_admin(qctx: &QueryContext) -> EnterpriseVersioningResult<AdminResponse> {
    let admin = ADMIN.load(qctx.deps.storage)?;

    Ok(AdminResponse { admin })
}

pub fn query_version(
    qctx: &QueryContext,
    params: VersionParams,
) -> EnterpriseVersioningResult<VersionResponse> {
    let version = VERSIONS.may_load(qctx.deps.storage, params.version)?;

    match version {
        None => Err(VersionNotFound {
            version: params.version,
        }),
        Some(version) => Ok(VersionResponse { version }),
    }
}

pub fn query_versions(
    qctx: &QueryContext,
    params: VersionsParams,
) -> EnterpriseVersioningResult<VersionsResponse> {
    let limit = params
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT as u32)
        .min(MAX_QUERY_LIMIT as u32);
    let start_after = params.start_after.map(Bound::exclusive);

    let versions = VERSIONS
        .range(qctx.deps.storage, start_after, None, Ascending)
        .take(limit as usize)
        .map(|res| res.map(|(_, version)| version))
        .collect::<StdResult<Vec<Version>>>()?;

    Ok(VersionsResponse { versions })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> EnterpriseVersioningResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
