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

pub fn execute_execute_msg_reply_callback_response() -> Response {
    Response::new().add_attribute("action", "execute_msg_reply_callback")
}
