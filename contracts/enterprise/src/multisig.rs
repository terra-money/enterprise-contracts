use cosmwasm_std::{Addr, BlockInfo, StdResult, Storage, Timestamp, Uint128};
use cw_storage_plus::{Map, SnapshotItem, Strategy};

pub const MULTISIG_MEMBERS: Map<Addr, Uint128> = Map::new("multisig_members");

// TODO: create a structure that holds both of those at the same time
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

pub fn load_total_multisig_weight(store: &dyn Storage) -> StdResult<Uint128> {
    TOTAL_MULTISIG_WEIGHT_AT_SECONDS.load(store)
}

pub fn load_total_multisig_weight_at_time(
    store: &dyn Storage,
    time: Timestamp,
) -> StdResult<Uint128> {
    Ok(TOTAL_MULTISIG_WEIGHT_AT_SECONDS
        .may_load_at_height(store, time.seconds())?
        .unwrap_or_default())
}

pub fn load_total_multisig_weight_at_height(
    store: &dyn Storage,
    height: u64,
) -> StdResult<Uint128> {
    Ok(TOTAL_MULTISIG_WEIGHT_AT_HEIGHT
        .may_load_at_height(store, height)?
        .unwrap_or_default())
}

pub fn save_total_multisig_weight(
    store: &mut dyn Storage,
    total_weight: Uint128,
    block: &BlockInfo,
) -> StdResult<()> {
    TOTAL_MULTISIG_WEIGHT_AT_SECONDS.save(store, &total_weight, block.time.seconds())?;
    TOTAL_MULTISIG_WEIGHT_AT_HEIGHT.save(store, &total_weight, block.height)?;

    Ok(())
}
