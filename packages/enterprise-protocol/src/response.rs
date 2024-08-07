use cosmwasm_std::Response;

pub fn instantiate_response() -> Response {
    Response::new().add_attribute("action", "instantiate")
}

pub fn execute_finalize_instantiation_response(
    attestation_contract: Option<String>,
    enterprise_governance_contract: String,
    enterprise_governance_controller_contract: String,
    enterprise_treasury_contract: String,
    funds_distributor_contract: String,
    membership_contract: String,
    council_membership_contract: String,
) -> Response {
    Response::new()
        .add_attribute("action", "finalize_instantiation")
        .add_attribute(
            "attestation_contract",
            attestation_contract.unwrap_or_else(|| "none".to_string()),
        )
        .add_attribute(
            "enterprise_governance_contract",
            enterprise_governance_contract,
        )
        .add_attribute(
            "enterprise_governance_controller_contract",
            enterprise_governance_controller_contract,
        )
        .add_attribute("enterprise_treasury_contract", enterprise_treasury_contract)
        .add_attribute("funds_distributor_contract", funds_distributor_contract)
        .add_attribute("membership_contract", membership_contract)
        .add_attribute("council_membership_contract", council_membership_contract)
}

pub fn execute_update_metadata_response() -> Response {
    Response::new().add_attribute("action", "update_metadata")
}

pub fn execute_upgrade_dao_response(new_dao_version: String) -> Response {
    Response::new()
        .add_attribute("action", "upgrade_dao")
        .add_attribute("new_version", new_dao_version)
}

pub fn execute_update_config_response(
    old_versioning_addr: String,
    new_versioning_addr: String,
    old_factory_addr: String,
    new_factory_addr: String,
) -> Response {
    Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("old_versioning_addr", old_versioning_addr)
        .add_attribute("new_versioning_addr", new_versioning_addr)
        .add_attribute("old_factory_addr", old_factory_addr)
        .add_attribute("new_factory_addr", new_factory_addr)
}

pub fn execute_set_attestation_response() -> Response {
    Response::new().add_attribute("action", "set_attestation")
}

pub fn execute_remove_attestation_response() -> Response {
    Response::new().add_attribute("action", "remove_attestation")
}

pub fn execute_execute_msgs_response() -> Response {
    Response::new().add_attribute("action", "execute_msgs")
}
