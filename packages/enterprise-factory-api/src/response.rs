use cosmwasm_std::Response;

pub fn instantiate_response() -> Response {
    Response::new().add_attribute("action", "instantiate")
}

pub fn execute_create_dao_response() -> Response {
    Response::new().add_attribute("action", "create_dao")
}

pub fn execute_finalize_dao_creation_response(
    enterprise_contract: String,
    enterprise_treasury_contract: String,
) -> Response {
    Response::new()
        .add_attribute("action", "finalize_dao_creation")
        .add_attribute("enterprise_contract", enterprise_contract)
        .add_attribute("enterprise_treasury_contract", enterprise_treasury_contract)
}
