use cosmwasm_std::Uint128;
use cw_storage_plus::Item;

// TODO: just bundle this with enterprise into Config?
/// Minimum weight that a user should have to be eligible for receiving rewards.
pub const MINIMUM_ELIGIBLE_WEIGHT: Item<Uint128> = Item::new("minimum_eligible_weight");
