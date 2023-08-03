use crate::facade_post_rewrite::EnterpriseFacadePostRewrite;
use crate::facade_v5::EnterpriseFacadeV5;
use common::cw::Context;
use cosmwasm_std::{Addr, Deps, Response, StdResult};
use enterprise_facade_api::api::{DaoInfoResponse, ProposalId};
use enterprise_facade_api::error::EnterpriseFacadeError::CannotCreateFacade;
use enterprise_facade_api::error::EnterpriseFacadeResult;
use enterprise_facade_api::msg::QueryMsg::DaoInfo;
use enterprise_treasury_api::api::ConfigResponse;
use enterprise_treasury_api::msg::QueryMsg::Config;

pub trait EnterpriseFacade {
    fn execute_proposal(
        &self,
        ctx: &mut Context,
        proposal_id: ProposalId,
    ) -> EnterpriseFacadeResult<Response>;
}

/// Get the correct facade implementation for the given address.
/// Address given will be for different contracts depending on Enterprise version.
/// For v0.5.0 (pre-rewrite) Enterprise, the address will be that of the enterprise contract itself.
/// For v1.0.0 (post-rewrite) Enterprise, the address will be that of the enterprise treasury contract.
pub fn get_facade(deps: Deps, address: Addr) -> EnterpriseFacadeResult<Box<dyn EnterpriseFacade>> {
    // attempt to query for DAO info
    let dao_info: StdResult<DaoInfoResponse> = deps
        .querier
        .query_wasm_smart(address.to_string(), &DaoInfo {});

    if dao_info.is_ok() {
        // if the query was successful, then this is a v0.5.0 (pre-rewrite) Enterprise contract
        Ok(Box::new(EnterpriseFacadeV5 {
            enterprise_address: address,
        }))
    } else {
        // if the query failed, this should be the post-rewrite Enterprise treasury, but let's check
        let treasury_config: StdResult<ConfigResponse> = deps
            .querier
            .query_wasm_smart(address.to_string(), &Config {});

        if treasury_config.is_ok() {
            Ok(Box::new(EnterpriseFacadePostRewrite {
                enterprise_treasury_address: address,
            }))
        } else {
            Err(CannotCreateFacade)
        }
    }
}
