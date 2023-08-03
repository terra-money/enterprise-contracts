use crate::facade::EnterpriseFacade;
use common::cw::Context;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{wasm_execute, Addr, Response, SubMsg};
use enterprise_facade_api::api::ProposalId;
use enterprise_facade_api::error::EnterpriseFacadeResult;

/// Facade implementation for v0.5.0 of Enterprise (pre-contract-rewrite).
pub struct EnterpriseFacadeV5 {
    pub enterprise_address: Addr,
}

impl EnterpriseFacade for EnterpriseFacadeV5 {
    fn execute_proposal(
        &self,
        _ctx: &mut Context,
        proposal_id: ProposalId,
    ) -> EnterpriseFacadeResult<Response> {
        let submsg = SubMsg::new(wasm_execute(
            self.enterprise_address.to_string(),
            &ExecuteV5Msg::ExecuteProposal(ExecuteProposalV5Msg { proposal_id }),
            vec![],
        )?);
        Ok(Response::new().add_submessage(submsg))
    }
}

#[cw_serde]
pub enum ExecuteV5Msg {
    ExecuteProposal(ExecuteProposalV5Msg),
}

#[cw_serde]
pub struct ExecuteProposalV5Msg {
    pub proposal_id: ProposalId,
}
