use common::cw::Context;
use cosmwasm_std::{StdResult, Uint128};
use cw_storage_plus::Item;

pub const TOTAL_STAKED: Item<Uint128> = Item::new("total_staked");

pub fn increment_total_staked(ctx: &mut Context) -> StdResult<Uint128> {
    let total_staked = TOTAL_STAKED.load(ctx.deps.storage)?;
    let new_total_staked = total_staked + Uint128::one();
    TOTAL_STAKED.save(ctx.deps.storage, &new_total_staked)?;

    Ok(new_total_staked)
}

pub fn decrement_total_staked(ctx: &mut Context, amount: Uint128) -> StdResult<Uint128> {
    let total_staked = TOTAL_STAKED.load(ctx.deps.storage)?;
    let new_total_staked = total_staked - amount;
    TOTAL_STAKED.save(ctx.deps.storage, &new_total_staked)?;

    Ok(new_total_staked)
}
