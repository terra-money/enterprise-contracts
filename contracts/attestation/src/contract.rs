use crate::state::{ATTESTATION_TEXT, USER_SIGNATURES};
use attestation_api::api::{AttestationTextResponse, HasUserSignedParams, HasUserSignedResponse};
use attestation_api::error::AttestationResult;
use attestation_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use attestation_api::response::{execute_sign_response, instantiate_response};
use common::cw::{Context, QueryContext};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
};
use cw2::set_contract_version;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:attestation";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> AttestationResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ATTESTATION_TEXT.save(deps.storage, &msg.attestation_text)?;

    Ok(instantiate_response())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> AttestationResult<Response> {
    let ctx = &mut Context { deps, env, info };

    match msg {
        ExecuteMsg::SignAttestation {} => sign_attestation(ctx),
    }
}

fn sign_attestation(ctx: &mut Context) -> AttestationResult<Response> {
    USER_SIGNATURES.save(ctx.deps.storage, ctx.info.sender.clone(), &())?;

    // TODO: report this change somehow to funds distributor and/or governance controller
    Ok(execute_sign_response(ctx.info.sender.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> AttestationResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> AttestationResult<Binary> {
    let qctx = QueryContext { deps, env };

    let response = match msg {
        QueryMsg::AttestationText {} => to_json_binary(&query_attestation_text(qctx)?)?,
        QueryMsg::HasUserSigned(params) => to_json_binary(&query_has_user_signed(qctx, params)?)?,
    };

    Ok(response)
}

fn query_attestation_text(qctx: QueryContext) -> AttestationResult<AttestationTextResponse> {
    let attestation_text = ATTESTATION_TEXT.load(qctx.deps.storage)?;

    Ok(AttestationTextResponse {
        text: attestation_text,
    })
}

fn query_has_user_signed(
    qctx: QueryContext,
    params: HasUserSignedParams,
) -> AttestationResult<HasUserSignedResponse> {
    let user = qctx.deps.api.addr_validate(&params.user)?;

    let has_signed = USER_SIGNATURES.has(qctx.deps.storage, user);

    Ok(HasUserSignedResponse { has_signed })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> AttestationResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
