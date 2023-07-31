use crate::enterprise_contract::ENTERPRISE_CONTRACT;
use common::cw::Context;
use cosmwasm_std::{Addr, Deps};
use enterprise_protocol::api::{
    ComponentContractsResponse, IsRestrictedUserParams, IsRestrictedUserResponse,
};
use enterprise_protocol::msg::QueryMsg::{ComponentContracts, IsRestrictedUser};
use membership_common_api::error::MembershipError::Unauthorized;
use membership_common_api::error::{MembershipError, MembershipResult};
use MembershipError::RestrictedUser;

pub fn validate_user_not_restricted(deps: Deps, user: String) -> MembershipResult<()> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(deps.storage)?;

    let response: IsRestrictedUserResponse = deps.querier.query_wasm_smart(
        enterprise_contract.to_string(),
        &IsRestrictedUser(IsRestrictedUserParams { user }),
    )?;

    if response.is_restricted {
        Err(RestrictedUser)
    } else {
        Ok(())
    }
}

/// Assert that the caller is admin.
/// If the validation succeeds, returns the admin address.
pub fn enterprise_governance_controller_only(
    ctx: &Context,
    sender: Option<String>,
) -> MembershipResult<Addr> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

    let component_contracts: ComponentContractsResponse = ctx
        .deps
        .querier
        .query_wasm_smart(enterprise_contract.to_string(), &ComponentContracts {})?;

    let governance_controller = component_contracts.enterprise_governance_controller_contract;

    let sender = sender
        .map(|addr| ctx.deps.api.addr_validate(&addr))
        .transpose()?
        .unwrap_or(ctx.info.sender.clone());

    // only governance controller contract is allowed
    if sender != governance_controller {
        return Err(Unauthorized);
    }

    Ok(governance_controller)
}
