use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cw_storage_plus::Map;

const USER_STAKES: Map<Addr, Uint128> = Map::new("user_stakes");

pub fn get_user_stake(storage: &dyn Storage, user: Addr) -> StdResult<Uint128> {
    Ok(USER_STAKES.may_load(storage, user)?.unwrap_or_default())
}

pub fn increment_user_stake(
    storage: &mut dyn Storage,
    user: Addr,
    amount: Uint128,
) -> StdResult<Uint128> {
    let user_stake = USER_STAKES
        .may_load(storage, user.clone())?
        .unwrap_or_default();
    let new_user_stake = user_stake + amount;
    USER_STAKES.save(storage, user, &(new_user_stake))?;

    Ok(new_user_stake)
}

pub fn decrement_user_total_staked(
    storage: &mut dyn Storage,
    user: Addr,
    amount: Uint128,
) -> StdResult<Uint128> {
    let user_stake = USER_STAKES
        .may_load(storage, user.clone())?
        .unwrap_or_default();
    let new_user_stake = user_stake - amount;
    USER_STAKES.save(storage, user, &(new_user_stake))?;

    Ok(new_user_stake)
}
