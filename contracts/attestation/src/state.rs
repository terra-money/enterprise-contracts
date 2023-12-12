use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const ATTESTATION_TEXT: Item<String> = Item::new("attestation_text");

pub const USER_SIGNATURES: Map<Addr, ()> = Map::new("user_signatures");
