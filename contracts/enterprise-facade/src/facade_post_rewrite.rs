use crate::facade::EnterpriseFacade;
use common::cw::Context;
use cosmwasm_std::{wasm_execute, Addr, Response, SubMsg};
use enterprise_facade_api::api::ProposalId;
use enterprise_facade_api::error::EnterpriseFacadeResult;
use enterprise_governance_controller_api::api::ExecuteProposalMsg;
use enterprise_governance_controller_api::msg::ExecuteMsg::ExecuteProposal;
use enterprise_protocol::api::ComponentContractsResponse;
use enterprise_protocol::msg::QueryMsg::ComponentContracts;
use enterprise_treasury_api::api::ConfigResponse;

/// Facade implementation for v1.0.0 of Enterprise (post-contract-rewrite).
pub struct EnterpriseFacadePostRewrite {
    pub enterprise_treasury_address: Addr,
}

impl EnterpriseFacade for EnterpriseFacadePostRewrite {
    fn execute_proposal(
        &self,
        ctx: &mut Context,
        proposal_id: ProposalId,
    ) -> EnterpriseFacadeResult<Response> {
        let treasury_config: ConfigResponse = ctx.deps.querier.query_wasm_smart(
            self.enterprise_treasury_address.to_string(),
            &enterprise_treasury_api::msg::QueryMsg::Config {},
        )?;
        let enterprise_contract = treasury_config.enterprise_contract;

        let component_contracts: ComponentContractsResponse = ctx
            .deps
            .querier
            .query_wasm_smart(enterprise_contract.to_string(), &ComponentContracts {})?;

        let governance_controller_contract =
            component_contracts.enterprise_governance_controller_contract;
        let submsg = SubMsg::new(wasm_execute(
            governance_controller_contract.to_string(),
            &ExecuteProposal(ExecuteProposalMsg { proposal_id }),
            vec![],
        )?);

        Ok(Response::new().add_submessage(submsg))
    }
}
