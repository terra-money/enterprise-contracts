use cosmwasm_std::Response;

pub fn instantiate_response(admin: String) -> Response {
    Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", admin)
}

pub fn execute_update_admin_response(old_admin: String, new_admin: String) -> Response {
    Response::new()
        .add_attribute("action", "update_admin")
        .add_attribute("old_admin", old_admin)
        .add_attribute("new_admin", new_admin)
}

pub fn execute_add_version_response(version: String) -> Response {
    Response::new()
        .add_attribute("action", "add_version")
        .add_attribute("version", version)
}

pub fn execute_edit_version_response(version: String) -> Response {
    Response::new()
        .add_attribute("action", "edit_version")
        .add_attribute("version", version)
}
