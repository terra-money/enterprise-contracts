use cosmwasm_std::{Addr, Deps, StdResult, Storage, Timestamp, Uint128};
use cw_storage_plus::{Map, SnapshotItem, Strategy};
use membership_common_api::api::TotalWeightCheckpoint;

pub const CW20_STAKES: Map<Addr, Uint128> = Map::new("stakes");

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

const TOTAL_MULTISIG_WEIGHT_AT_SECONDS: SnapshotItem<Uint128> = SnapshotItem::new(
    "total_multisig_weight_seconds",
    "total_multisig_weight_checkpoints_seconds",
    "total_multisig_weight_changelog_seconds",
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

pub fn get_seconds_checkpoints(
    deps: Deps,
    timestamps_to_get: Vec<Timestamp>,
) -> StdResult<Vec<TotalWeightCheckpoint>> {
    load_timestamp_total_weights(deps, TOTAL_STAKED_SECONDS_SNAPSHOT, timestamps_to_get)
}

pub fn get_multisig_seconds_checkpoints(
    deps: Deps,
    timestamps_to_get: Vec<Timestamp>,
) -> StdResult<Vec<TotalWeightCheckpoint>> {
    load_timestamp_total_weights(deps, TOTAL_MULTISIG_WEIGHT_AT_SECONDS, timestamps_to_get)
}

pub fn load_timestamp_total_weights(
    deps: Deps,
    snapshot_item: SnapshotItem<Uint128>,
    timestamps: Vec<Timestamp>,
) -> StdResult<Vec<TotalWeightCheckpoint>> {
    let checkpoints = timestamps
        .into_iter()
        .map(|timestamp| {
            snapshot_item
                .may_load_at_height(deps.storage, timestamp.seconds())
                .map(|opt_weight| TotalWeightCheckpoint {
                    height: timestamp.seconds(),
                    total_weight: opt_weight.unwrap_or_default(),
                })
        })
        .collect::<StdResult<Vec<TotalWeightCheckpoint>>>()?;

    Ok(checkpoints)
}
