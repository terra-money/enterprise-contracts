use cosmwasm_std::{BlockInfo, StdResult, Storage, Timestamp, Uint128};
use cw_storage_plus::{SnapshotItem, Strategy};

const TOTAL_WEIGHT_HEIGHT_SNAPSHOT: SnapshotItem<Uint128> = SnapshotItem::new(
    "total_weight_block_height_snapshot",
    "total_weight_block_height_checkpoints",
    "total_weight_block_height_changelog",
    Strategy::EveryBlock,
);
const TOTAL_WEIGHT_SECONDS_SNAPSHOT: SnapshotItem<Uint128> = SnapshotItem::new(
    "total_weight_time_seconds_snapshot",
    "total_weight_time_seconds_checkpoints",
    "total_weight_time_seconds_changelog",
    Strategy::EveryBlock,
);

pub fn load_total_weight(store: &dyn Storage) -> StdResult<Uint128> {
    TOTAL_WEIGHT_HEIGHT_SNAPSHOT.load(store)
}

pub fn load_total_weight_at_height(store: &dyn Storage, height: u64) -> StdResult<Uint128> {
    Ok(TOTAL_WEIGHT_HEIGHT_SNAPSHOT
        .may_load_at_height(store, height)?
        .unwrap_or_default())
}

pub fn load_total_weight_at_time(store: &dyn Storage, time: Timestamp) -> StdResult<Uint128> {
    Ok(TOTAL_WEIGHT_SECONDS_SNAPSHOT
        .may_load_at_height(store, time.seconds())?
        .unwrap_or_default())
}

pub fn save_total_weight(
    store: &mut dyn Storage,
    amount: &Uint128,
    block: &BlockInfo,
) -> StdResult<()> {
    TOTAL_WEIGHT_HEIGHT_SNAPSHOT.save(store, amount, block.height)?;
    TOTAL_WEIGHT_SECONDS_SNAPSHOT.save(store, amount, block.time.seconds())?;

    Ok(())
}
