use crate::facade::QueryV1Msg::DaoInfo;
use crate::state::{ENTERPRISE_FACADE_V1, ENTERPRISE_FACADE_V2};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, StdResult, Timestamp, Uint64};
use enterprise_facade_api::api::{DaoCouncil, DaoMetadata, DaoType, GovConfigV1};
use enterprise_facade_api::error::EnterpriseFacadeError::CannotCreateFacade;
use enterprise_facade_api::error::EnterpriseFacadeResult;
use enterprise_protocol::api::ComponentContractsResponse;
use enterprise_treasury_api::api::ConfigResponse;
use enterprise_treasury_api::msg::QueryMsg::Config;

#[cw_serde]
pub enum QueryV1Msg {
    DaoInfo {},
}

#[cw_serde]
pub struct DaoInfoResponseV1 {
    pub creation_date: Timestamp,
    pub metadata: DaoMetadata,
    pub gov_config: GovConfigV1,
    pub dao_council: Option<DaoCouncil>,
    pub dao_type: DaoType,
    pub dao_membership_contract: Addr,
    pub enterprise_factory_contract: Addr,
    pub funds_distributor_contract: Addr,
    pub dao_code_version: Uint64,
}

/// Structure informing us the address of the facade contract to use, and the DAO address to use
/// when calling the facade.
#[cw_serde]
pub struct FacadeTarget {
    pub facade_address: Addr,
    pub dao_address: Addr,
}

/// Get the correct facade implementation for the given address.
/// Address given will be for different contracts depending on Enterprise version.
/// For v0.5.0 (pre-rewrite) Enterprise, the address will be that of the enterprise contract itself.
/// For v1.0.0 (post-rewrite) Enterprise, the address will be that of the enterprise treasury contract or the enterprise contract.
pub fn get_facade(deps: Deps, address: Addr) -> EnterpriseFacadeResult<FacadeTarget> {
    // attempt to query for DAO info
    let dao_info: StdResult<DaoInfoResponseV1> = deps
        .querier
        .query_wasm_smart(address.to_string(), &DaoInfo {});

    if dao_info.is_ok() {
        // if the query was successful, then this is a v0.5.0 (pre-rewrite) enterprise contract
        let facade_v1 = ENTERPRISE_FACADE_V1.load(deps.storage)?;
        Ok(FacadeTarget {
            facade_address: facade_v1,
            dao_address: address,
        })
    } else {
        // if the query failed, this should be either the post-rewrite enterprise-treasury or enterprise contract, but let's check

        if let Ok(facade) =
            get_facade_assume_post_rewrite_enterprise_treasury(deps, address.clone())
        {
            Ok(facade)
        } else {
            get_facade_assume_post_rewrite_enterprise(deps, address)
        }
    }
}

fn get_facade_assume_post_rewrite_enterprise_treasury(
    deps: Deps,
    treasury_address: Addr,
) -> EnterpriseFacadeResult<FacadeTarget> {
    let treasury_config: ConfigResponse = deps
        .querier
        .query_wasm_smart(treasury_address.to_string(), &Config {})
        .map_err(|_| CannotCreateFacade)?;

    let governance_controller_config: enterprise_governance_controller_api::api::ConfigResponse =
        deps.querier
            .query_wasm_smart(
                treasury_config.admin.to_string(),
                &enterprise_governance_controller_api::msg::QueryMsg::Config {},
            )
            .map_err(|_| CannotCreateFacade)?;

    let facade_v2 = ENTERPRISE_FACADE_V2.load(deps.storage)?;

    Ok(FacadeTarget {
        facade_address: facade_v2,
        dao_address: governance_controller_config.enterprise_contract,
    })
}

fn get_facade_assume_post_rewrite_enterprise(
    deps: Deps,
    enterprise_address: Addr,
) -> EnterpriseFacadeResult<FacadeTarget> {
    let _: ComponentContractsResponse = deps
        .querier
        .query_wasm_smart(
            enterprise_address.to_string(),
            &enterprise_protocol::msg::QueryMsg::ComponentContracts {},
        )
        .map_err(|_| CannotCreateFacade)?;

    let facade_v2 = ENTERPRISE_FACADE_V2.load(deps.storage)?;

    Ok(FacadeTarget {
        facade_address: facade_v2,
        dao_address: enterprise_address,
    })
}
