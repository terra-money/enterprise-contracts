use cosmwasm_std::{Response, Uint128};

pub fn instantiate_response(admin: String) -> Response {
    Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", admin)
}

pub fn execute_update_user_weights_response() -> Response {
    Response::new().add_attribute("action", "update_user_weights")
}

pub fn execute_pre_user_votes_change_response() -> Response {
    Response::new().add_attribute("action", "pre_user_votes_change")
}

pub fn execute_new_proposal_created_response(proposal_id: u64) -> Response {
    Response::new()
        .add_attribute("action", "new_proposal_created")
        .add_attribute("proposal_id", proposal_id.to_string())
}

pub fn execute_update_minimum_eligible_weight_response(
    old_minimum_weight: Uint128,
    new_minimum_weight: Uint128,
) -> Response {
    Response::new()
        .add_attribute("action", "update_minimum_eligible_weight")
        .add_attribute("old_minimum_weight", old_minimum_weight.to_string())
        .add_attribute("new_minimum_weight", new_minimum_weight.to_string())
}

pub fn execute_update_number_proposals_tracked_response(
    new_number_tracked: u8,
) -> Response {
    Response::new()
        .add_attribute("action", "update_number_proposals_tracked")
        .add_attribute("new_number_tracked", new_number_tracked.to_string())
}

pub fn execute_distribute_native_response(total_weight: Uint128) -> Response {
    Response::new()
        .add_attribute("action", "distribute_native")
        .add_attribute("total_weight", total_weight.to_string())
}

pub fn execute_claim_rewards_response(user: String) -> Response {
    Response::new()
        .add_attribute("action", "claim_rewards")
        .add_attribute("user", user)
}

pub fn cw20_hook_distribute_cw20_response(
    total_weight: Uint128,
    cw20_asset: String,
    amount: Uint128,
) -> Response {
    Response::new()
        .add_attribute("action", "distribute_cw20")
        .add_attribute("total_weight", total_weight.to_string())
        .add_attribute("cw20_asset", cw20_asset)
        .add_attribute("amount_distributed", amount.to_string())
}
