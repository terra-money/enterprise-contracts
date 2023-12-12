use cosmwasm_std::Response;

pub fn instantiate_response() -> Response {
    Response::new().add_attribute("action", "instantiate")
}

pub fn execute_sign_response(user: String) -> Response {
    Response::new()
        .add_attribute("action", "sign")
        .add_attribute("user", user)
}
