use crate::claim::claim_rewards;
use crate::distributing::{distribute_cw20, distribute_native};
use crate::eligibility::{
    execute_update_minimum_eligible_weight, query_minimum_eligible_weight, MINIMUM_ELIGIBLE_WEIGHT,
};
use crate::participation::{
    query_number_proposals_tracked, query_proposal_ids_tracked, PROPOSALS_TRACKED,
};
use crate::rewards::query_user_rewards;
use crate::state::{ADMIN, ENTERPRISE_CONTRACT};
use crate::user_weights::{save_initial_weights, update_user_weights};
use common::cw::{Context, QueryContext};
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdError,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use funds_distributor_api::error::DistributorResult;
use funds_distributor_api::msg::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use funds_distributor_api::response::instantiate_response;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:funds-distributor";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> DistributorResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = deps.api.addr_validate(&msg.admin)?;
    ADMIN.save(deps.storage, &admin)?;

    let enterprise_contract = deps.api.addr_validate(&msg.enterprise_contract)?;
    ENTERPRISE_CONTRACT.save(deps.storage, &enterprise_contract)?;

    let minimum_eligible_weight = msg.minimum_eligible_weight.unwrap_or_default();
    MINIMUM_ELIGIBLE_WEIGHT.save(deps.storage, &minimum_eligible_weight)?;

    PROPOSALS_TRACKED.save(
        deps.storage,
        &msg.participation_proposals_tracked.unwrap_or_default(),
    )?;

    let mut ctx = Context { deps, env, info };

    save_initial_weights(&mut ctx, msg.initial_weights)?;

    Ok(instantiate_response(admin.to_string()))
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
        ExecuteMsg::UpdateMinimumEligibleWeight(msg) => {
            execute_update_minimum_eligible_weight(ctx, msg)
        }
        ExecuteMsg::DistributeNative { distribution_type } => {
            distribute_native(ctx, distribution_type)
        }
        ExecuteMsg::ClaimRewards(msg) => claim_rewards(ctx, msg),
        ExecuteMsg::Receive(msg) => receive_cw20(ctx, msg),
    }
}

fn receive_cw20(ctx: &mut Context, cw20_msg: Cw20ReceiveMsg) -> DistributorResult<Response> {
    match from_json(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Distribute { distribution_type }) => {
            distribute_cw20(ctx, cw20_msg, distribution_type)
        }
        _ => Err(StdError::generic_err("Received unknown CW20 hook message").into()),
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
        QueryMsg::UserRewards(params) => to_json_binary(&query_user_rewards(qctx, params)?)?,
        QueryMsg::MinimumEligibleWeight {} => {
            to_json_binary(&query_minimum_eligible_weight(qctx)?)?
        }
        QueryMsg::NumberProposalsTracked {} => {
            to_json_binary(&query_number_proposals_tracked(qctx)?)?
        }
        QueryMsg::ProposalIdsTracked {} => to_json_binary(&query_proposal_ids_tracked(qctx)?)?,
    };
    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> DistributorResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
