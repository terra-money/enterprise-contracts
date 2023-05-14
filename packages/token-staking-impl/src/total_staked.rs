use common::cw::Context;
use cosmwasm_std::{BlockInfo, StdResult, Storage, Timestamp, Uint128};
use cw_storage_plus::{SnapshotItem, Strategy};

pub fn increment_total_staked(ctx: &mut Context, amount: Uint128) -> StdResult<Uint128> {
    let total_staked = load_total_staked(ctx.deps.storage)?;
    let new_total_staked = total_staked + amount;
    save_total_staked(ctx.deps.storage, &new_total_staked, &ctx.env.block)?;

    Ok(new_total_staked)
}

pub fn decrement_total_staked(ctx: &mut Context, amount: Uint128) -> StdResult<Uint128> {
    let total_staked = load_total_staked(ctx.deps.storage)?;
    let new_total_staked = total_staked - amount;
    save_total_staked(ctx.deps.storage, &new_total_staked, &ctx.env.block)?;

    Ok(new_total_staked)
}

const TOTAL_STAKED_HEIGHT_SNAPSHOT: SnapshotItem<Uint128> = SnapshotItem::new(
    "total_staked_block_height_snapshot",
    "total_staked_block_height_checkpoints",
    "total_staked_block_height_changelog",
    Strategy::EveryBlock,
);
const TOTAL_STAKED_SECONDS_SNAPSHOT: SnapshotItem<Uint128> = SnapshotItem::new(
    "total_staked_time_seconds_snapshot",
    "total_staked_time_seconds_checkpoints",
    "total_staked_time_seconds_changelog",
    Strategy::EveryBlock,
);

pub fn load_total_staked(store: &dyn Storage) -> StdResult<Uint128> {
    TOTAL_STAKED_HEIGHT_SNAPSHOT.load(store)
}

pub fn load_total_staked_at_height(store: &dyn Storage, height: u64) -> StdResult<Uint128> {
    Ok(TOTAL_STAKED_HEIGHT_SNAPSHOT
        .may_load_at_height(store, height)?
        .unwrap_or_default())
}

pub fn load_total_staked_at_time(store: &dyn Storage, time: Timestamp) -> StdResult<Uint128> {
    Ok(TOTAL_STAKED_SECONDS_SNAPSHOT
        .may_load_at_height(store, time.seconds())?
        .unwrap_or_default())
}

pub fn save_total_staked(
    store: &mut dyn Storage,
    amount: &Uint128,
    block: &BlockInfo,
) -> StdResult<()> {
    TOTAL_STAKED_HEIGHT_SNAPSHOT.save(store, amount, block.height)?;
    TOTAL_STAKED_SECONDS_SNAPSHOT.save(store, amount, block.time.seconds())?;

    Ok(())
}
