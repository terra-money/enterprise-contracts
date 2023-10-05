use crate::state::COMPONENT_CONTRACTS;
use common::cw::Context;
use enterprise_protocol::error::DaoError::Unauthorized;
use enterprise_protocol::error::DaoResult;

/// Asserts that the caller is enterprise-governance-controller contract.
pub fn enterprise_governance_controller_caller_only(ctx: &Context) -> DaoResult<()> {
    let component_contracts = COMPONENT_CONTRACTS.load(ctx.deps.storage)?;

    if ctx.info.sender != component_contracts.enterprise_governance_controller_contract {
        Err(Unauthorized)
    } else {
        Ok(())
    }
}
