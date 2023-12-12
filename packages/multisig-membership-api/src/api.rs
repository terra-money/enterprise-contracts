use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct UserWeight {
    pub user: String,
    pub weight: Uint128,
}

#[cw_serde]
pub struct UpdateMembersMsg {
    /// Members to be updated.
    /// Can contain existing members, in which case their new weight will be the one specified in
    /// this message. This effectively allows removing of members (by setting their weight to 0).
    pub update_members: Vec<UserWeight>,
}

#[cw_serde]
pub struct SetMembersMsg {
    /// All existing members will be removed, and replaced with the given members and their weights.
    pub new_members: Vec<UserWeight>,
}

// Responses

#[cw_serde]
pub struct ConfigResponse {
    pub enterprise_contract: Addr,
}
