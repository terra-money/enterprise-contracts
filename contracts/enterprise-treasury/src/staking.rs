use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, Deps, StdResult, Uint128};
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

const TOTAL_MULTISIG_WEIGHT_AT_HEIGHT: SnapshotItem<Uint128> = SnapshotItem::new(
    "total_multisig_weight_height",
    "total_multisig_weight_checkpoints_height",
    "total_multisig_weight_changelog_height",
    Strategy::EveryBlock,
);

pub fn get_seconds_checkpoints(deps: Deps) -> StdResult<Vec<TotalWeightCheckpoint>> {
    get_checkpoints(deps, TOTAL_STAKED_SECONDS_SNAPSHOT)
}

pub fn get_height_checkpoints(deps: Deps) -> StdResult<Vec<TotalWeightCheckpoint>> {
    get_checkpoints(deps, TOTAL_STAKED_HEIGHT_SNAPSHOT)
}

pub fn get_multisig_seconds_checkpoints(deps: Deps) -> StdResult<Vec<TotalWeightCheckpoint>> {
    get_checkpoints(deps, TOTAL_MULTISIG_WEIGHT_AT_SECONDS)
}

pub fn get_multisig_height_checkpoints(deps: Deps) -> StdResult<Vec<TotalWeightCheckpoint>> {
    get_checkpoints(deps, TOTAL_MULTISIG_WEIGHT_AT_HEIGHT)
}

pub fn get_checkpoints(
    deps: Deps,
    snapshot_item: SnapshotItem<Uint128>,
) -> StdResult<Vec<TotalWeightCheckpoint>> {
    let mut checkpoints = vec![];

    let mut last_seen_key: Option<u64> = None;

    for change_key_res in snapshot_item
        .changelog()
        .keys(deps.storage, None, None, Ascending)
    {
        let change_key = change_key_res?;

        if let Some(last_seen_key) = last_seen_key {
            let old_value = snapshot_item
                .changelog()
                .may_load(deps.storage, change_key)?
                .and_then(|changeset| changeset.old);
            if let Some(total_weight) = old_value {
                checkpoints.push(TotalWeightCheckpoint {
                    height: last_seen_key,
                    total_weight,
                });
            }
        }

        last_seen_key = Some(change_key);
    }

    // we need to add the latest known weight at the last checkpoint height we encountered
    if let Some(key) = last_seen_key {
        let current_total_weight = snapshot_item.load(deps.storage)?;
        checkpoints.push(TotalWeightCheckpoint {
            height: key,
            total_weight: current_total_weight,
        });
    }

    Ok(checkpoints)
}
