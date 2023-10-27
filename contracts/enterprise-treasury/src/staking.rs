use cosmwasm_std::{Addr, StdResult, Storage, Timestamp, Uint128};
use cw_storage_plus::{Map, SnapshotItem, Strategy};

pub const CW20_STAKES: Map<Addr, Uint128> = Map::new("stakes");

// TODO: uses double the storage, can we avoid this somehow? we do need timestamp snapshots after all
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
