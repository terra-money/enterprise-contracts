use cosmwasm_std::Response;

pub fn instantiate_response(admin: String) -> Response {
    Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", admin)
}

pub fn execute_set_admin_response(new_admin: String) -> Response {
    Response::new()
        .add_attribute("action", "set_admin")
        .add_attribute("new_admin", new_admin)
}

pub fn execute_update_asset_whitelist_response() -> Response {
    Response::new().add_attribute("action", "update_asset_whitelist")
}

pub fn execute_update_nft_whitelist_response() -> Response {
    Response::new().add_attribute("action", "update_nft_whitelist")
}

pub fn execute_spend_response() -> Response {
    Response::new().add_attribute("action", "spend")
}

pub fn execute_distribute_funds_response() -> Response {
    Response::new().add_attribute("action", "distribute_funds")
}

pub fn execute_execute_cosmos_msgs_response() -> Response {
    Response::new().add_attribute("action", "execute_cosmos_msgs")
}
