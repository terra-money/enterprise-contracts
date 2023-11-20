use cosmwasm_std::Response;

pub fn instantiate_response() -> Response {
    Response::new().add_attribute("action", "instantiate")
}

pub fn execute_deploy_cross_chain_proxy_response() -> Response {
    Response::new().add_attribute("action", "deploy_cross_chain_proxy")
}

pub fn execute_deploy_cross_chain_treasury_response() -> Response {
    Response::new().add_attribute("action", "deploy_cross_chain_treasury")
}

pub fn execute_execute_cross_chain_treasury_response() -> Response {
    Response::new().add_attribute("action", "execute_cross_chain_treasury")
}

pub fn execute_instantiate_proxy_reply_callback_response(
    dao_address: String,
    chain_id: String,
    proxy_address: String,
) -> Response {
    Response::new()
        .add_attribute("action", "instantiate_proxy_reply_callback")
        .add_attribute("dao_address", dao_address)
        .add_attribute("chain_id", chain_id)
        .add_attribute("proxy_address", proxy_address)
}

pub fn execute_instantiate_treasury_reply_callback_response(
    dao_address: String,
    chain_id: String,
    treasury_address: String,
) -> Response {
    Response::new()
        .add_attribute("action", "instantiate_treasury_reply_callback")
        .add_attribute("dao_address", dao_address)
        .add_attribute("chain_id", chain_id)
        .add_attribute("treasury_address", treasury_address)
}
