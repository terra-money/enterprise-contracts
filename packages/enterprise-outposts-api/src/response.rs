use cosmwasm_std::Response;

pub fn instantiate_response() -> Response {
    Response::new().add_attribute("action", "instantiate")
}

pub fn execute_add_cross_chain_proxy_response() -> Response {
    Response::new().add_attribute("action", "add_cross_chain_proxy")
}

pub fn execute_add_cross_chain_treasury_response() -> Response {
    Response::new().add_attribute("action", "add_cross_chain_treasury")
}
