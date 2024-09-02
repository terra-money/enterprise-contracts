use crate::state::{ADMIN, VERSIONS};
use common::cw::{Context, QueryContext};
use cosmwasm_std::Order::{Ascending, Descending};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use enterprise_versioning_api::api::{
    AddVersionMsg, AdminResponse, EditVersionMsg, UpdateAdminMsg, VersionInfo, VersionParams,
    VersionResponse, VersionsParams, VersionsResponse,
};
use enterprise_versioning_api::error::EnterpriseVersioningError::{
    NoVersionsExist, Unauthorized, VersionAlreadyExists, VersionNotFound,
};
use enterprise_versioning_api::error::EnterpriseVersioningResult;
use enterprise_versioning_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use enterprise_versioning_api::response::{
    execute_add_version_response, execute_edit_version_response, execute_update_admin_response,
    instantiate_response,
};

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

    Ok(instantiate_response(admin.to_string()))
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
        ExecuteMsg::UpdateAdmin(msg) => update_admin(ctx, msg),
        ExecuteMsg::AddVersion(msg) => add_version(ctx, msg),
        ExecuteMsg::EditVersion(msg) => edit_version(ctx, msg),
    }
}

fn update_admin(ctx: &mut Context, msg: UpdateAdminMsg) -> EnterpriseVersioningResult<Response> {
    let admin = ADMIN.load(ctx.deps.storage)?;

    if ctx.info.sender != admin {
        return Err(Unauthorized);
    }

    let new_admin = ctx.deps.api.addr_validate(&msg.new_admin)?;

    ADMIN.save(ctx.deps.storage, &new_admin)?;

    Ok(execute_update_admin_response(
        admin.to_string(),
        new_admin.to_string(),
    ))
}

fn add_version(ctx: &mut Context, msg: AddVersionMsg) -> EnterpriseVersioningResult<Response> {
    let admin = ADMIN.load(ctx.deps.storage)?;

    if admin != ctx.info.sender {
        return Err(Unauthorized);
    }

    let version = msg.version.version.clone();
    let version_key = version.clone().into();

    if VERSIONS.has(ctx.deps.storage, version_key) {
        return Err(VersionAlreadyExists { version });
    }

    VERSIONS.save(ctx.deps.storage, version_key, &msg.version)?;

    Ok(execute_add_version_response(version.to_string()))
}

fn edit_version(ctx: &mut Context, msg: EditVersionMsg) -> EnterpriseVersioningResult<Response> {
    let admin = ADMIN.load(ctx.deps.storage)?;

    if admin != ctx.info.sender {
        return Err(Unauthorized);
    }

    let version_key = msg.version.clone().into();

    let version_info = VERSIONS.may_load(ctx.deps.storage, version_key)?;

    match version_info {
        None => {
            return Err(VersionNotFound {
                version: msg.version,
            });
        }
        Some(version_info) => {
            let edited_version = apply_edit_changes(version_info, &msg)?;
            VERSIONS.save(ctx.deps.storage, version_key, &edited_version)?;
        }
    }

    Ok(execute_edit_version_response(msg.version.to_string()))
}

fn apply_edit_changes(
    mut version_info: VersionInfo,
    msg: &EditVersionMsg,
) -> EnterpriseVersioningResult<VersionInfo> {
    if let Some(changelog) = &msg.changelog {
        version_info.changelog = changelog.clone();
    }

    if let Some(attestation_code_id) = msg.attestation_code_id {
        version_info.attestation_code_id = attestation_code_id;
    }

    if let Some(enterprise_code_id) = msg.enterprise_code_id {
        version_info.enterprise_code_id = enterprise_code_id;
    }

    if let Some(enterprise_governance_code_id) = msg.enterprise_governance_code_id {
        version_info.enterprise_governance_code_id = enterprise_governance_code_id;
    }

    if let Some(enterprise_governance_controller_code_id) =
        msg.enterprise_governance_controller_code_id
    {
        version_info.enterprise_governance_controller_code_id =
            enterprise_governance_controller_code_id;
    }

    if let Some(enterprise_outposts_code_id) = msg.enterprise_outposts_code_id {
        version_info.enterprise_outposts_code_id = enterprise_outposts_code_id;
    }

    if let Some(enterprise_treasury_code_id) = msg.enterprise_treasury_code_id {
        version_info.enterprise_treasury_code_id = enterprise_treasury_code_id;
    }

    if let Some(funds_distributor_code_id) = msg.funds_distributor_code_id {
        version_info.funds_distributor_code_id = funds_distributor_code_id;
    }

    if let Some(token_staking_membership_code_id) = msg.token_staking_membership_code_id {
        version_info.token_staking_membership_code_id = token_staking_membership_code_id;
    }

    if let Some(denom_staking_membership_code_id) = msg.denom_staking_membership_code_id {
        version_info.denom_staking_membership_code_id = denom_staking_membership_code_id;
    }

    if let Some(nft_staking_membership_code_id) = msg.nft_staking_membership_code_id {
        version_info.nft_staking_membership_code_id = nft_staking_membership_code_id;
    }

    if let Some(multisig_membership_code_id) = msg.multisig_membership_code_id {
        version_info.multisig_membership_code_id = multisig_membership_code_id;
    }

    Ok(version_info)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> EnterpriseVersioningResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> EnterpriseVersioningResult<Binary> {
    let qctx = QueryContext { deps, env };

    let response = match msg {
        QueryMsg::Admin {} => to_json_binary(&query_admin(&qctx)?)?,
        QueryMsg::Version(params) => to_json_binary(&query_version(&qctx, params)?)?,
        QueryMsg::Versions(params) => to_json_binary(&query_versions(&qctx, params)?)?,
        QueryMsg::LatestVersion {} => to_json_binary(&query_latest_version(&qctx)?)?,
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
    let version = VERSIONS.may_load(qctx.deps.storage, params.version.clone().into())?;

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
        .collect::<StdResult<Vec<VersionInfo>>>()?;

    Ok(VersionsResponse { versions })
}

pub fn query_latest_version(qctx: &QueryContext) -> EnterpriseVersioningResult<VersionResponse> {
    if VERSIONS.is_empty(qctx.deps.storage) {
        Err(NoVersionsExist)
    } else {
        let latest_version = VERSIONS
            .range(qctx.deps.storage, None, None, Descending)
            .take(1)
            .map(|res| res.map(|(_, version)| version))
            .collect::<StdResult<Vec<VersionInfo>>>()?
            .first()
            .cloned()
            .ok_or(NoVersionsExist)?;
        Ok(VersionResponse {
            version: latest_version,
        })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> EnterpriseVersioningResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
