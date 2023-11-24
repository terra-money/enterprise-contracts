use crate::staking::load_timestamp_total_weights;
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{Timestamp, Uint128};
use cw_storage_plus::{SnapshotItem, Strategy};
use enterprise_treasury_api::error::EnterpriseTreasuryResult;
use membership_common_api::api::TotalWeightCheckpoint;
use Strategy::EveryBlock;

#[test]
fn test_get_checkpoints_for_timestamps() -> EnterpriseTreasuryResult<()> {
    let mut deps = mock_dependencies();

    let snapshot_item: SnapshotItem<Uint128> =
        SnapshotItem::new("snapshot", "checkpoints", "changelog", EveryBlock);

    snapshot_item.save(deps.as_mut().storage, &Uint128::from(1u8), 20)?;
    snapshot_item.save(deps.as_mut().storage, &Uint128::from(2u8), 25)?;
    snapshot_item.save(deps.as_mut().storage, &Uint128::from(3u8), 30)?;
    snapshot_item.save(deps.as_mut().storage, &Uint128::from(4u8), 35)?;
    snapshot_item.save(deps.as_mut().storage, &Uint128::from(5u8), 40)?;

    let checkpoints = load_timestamp_total_weights(
        deps.as_ref(),
        snapshot_item,
        vec![
            Timestamp::from_seconds(22),
            Timestamp::from_seconds(29),
            Timestamp::from_seconds(35),
        ],
    )?;

    assert_eq!(
        checkpoints,
        vec![
            TotalWeightCheckpoint {
                height: 22,
                total_weight: Uint128::from(1u8)
            },
            TotalWeightCheckpoint {
                height: 29,
                total_weight: Uint128::from(2u8)
            },
            TotalWeightCheckpoint {
                height: 35,
                total_weight: Uint128::from(3u8)
            },
        ]
    );

    Ok(())
}
