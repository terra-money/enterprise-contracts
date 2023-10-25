use crate::state::ENTERPRISE_CONTRACT;
use common::cw::Context;
use enterprise_outposts_api::error::EnterpriseOutpostsError::Unauthorized;
use enterprise_outposts_api::error::EnterpriseOutpostsResult;
use enterprise_protocol::api::ComponentContractsResponse;
use enterprise_protocol::msg::QueryMsg::ComponentContracts;

/// Asserts that the caller is enterprise-governance-controller contract.
pub fn enterprise_governance_controller_caller_only(ctx: &Context) -> EnterpriseOutpostsResult<()> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

    let component_contracts: ComponentContractsResponse = ctx
        .deps
        .querier
        .query_wasm_smart(enterprise_contract.to_string(), &ComponentContracts {})?;

    if ctx.info.sender != component_contracts.enterprise_governance_controller_contract {
        Err(Unauthorized)
    } else {
        Ok(())
    }
}
