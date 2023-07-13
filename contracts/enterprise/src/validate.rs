use crate::state::{COMPONENT_CONTRACTS, ENTERPRISE_FACTORY_CONTRACT};
use common::cw::Context;
use enterprise_protocol::error::DaoError::Unauthorized;
use enterprise_protocol::error::DaoResult;

/// Asserts that the caller is enterprise-factory contract.
pub fn enterprise_factory_caller_only(ctx: &Context) -> DaoResult<()> {
    let enterprise_factory = ENTERPRISE_FACTORY_CONTRACT.load(ctx.deps.storage)?;

    if ctx.info.sender != enterprise_factory {
        Err(Unauthorized)
    } else {
        Ok(())
    }
}

/// Asserts that the caller is enterprise-governance-controller contract.
pub fn enterprise_governance_controller_caller_only(ctx: &Context) -> DaoResult<()> {
    let component_contracts = COMPONENT_CONTRACTS.load(ctx.deps.storage)?;

    if ctx.info.sender != component_contracts.enterprise_governance_controller_contract {
        Err(Unauthorized)
    } else {
        Ok(())
    }
}
