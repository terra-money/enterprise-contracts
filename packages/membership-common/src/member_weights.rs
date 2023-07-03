use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cw_storage_plus::Map;

pub const MEMBER_WEIGHTS: Map<Addr, Uint128> = Map::new("membership_common__member_weights");

pub fn get_member_weight(storage: &dyn Storage, member: Addr) -> StdResult<Uint128> {
    Ok(MEMBER_WEIGHTS
        .may_load(storage, member)?
        .unwrap_or_default())
}

pub fn set_member_weight(
    storage: &mut dyn Storage,
    member: Addr,
    amount: Uint128,
) -> StdResult<()> {
    MEMBER_WEIGHTS.save(storage, member, &amount)?;

    Ok(())
}

pub fn increment_member_weight(
    storage: &mut dyn Storage,
    member: Addr,
    amount: Uint128,
) -> StdResult<Uint128> {
    let member_weight = get_member_weight(storage, member.clone())?;
    let new_member_weight = member_weight + amount;
    set_member_weight(storage, member, new_member_weight)?;

    Ok(new_member_weight)
}

pub fn decrement_member_weight(
    storage: &mut dyn Storage,
    member: Addr,
    amount: Uint128,
) -> StdResult<Uint128> {
    let member_weight = get_member_weight(storage, member.clone())?;
    let new_member_weight = member_weight - amount;
    set_member_weight(storage, member, new_member_weight)?;

    Ok(new_member_weight)
}
