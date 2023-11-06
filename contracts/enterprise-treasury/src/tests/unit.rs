use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::Uint128;
use cw_storage_plus::{SnapshotItem, Strategy};
use Strategy::EveryBlock;
use enterprise_treasury_api::error::EnterpriseTreasuryResult;
use membership_common_api::api::TotalWeightCheckpoint;
use crate::staking::{get_checkpoints};

#[test]
fn test_get_checkpoints() -> EnterpriseTreasuryResult<()> {
    let mut deps = mock_dependencies();

    let snapshot_item: SnapshotItem<Uint128> = SnapshotItem::new(
        "snapshot",
        "checkpoints",
        "changelog",
        EveryBlock,
    );

    snapshot_item.save(deps.as_mut().storage, &Uint128::from(1u8), 20)?;
    snapshot_item.save(deps.as_mut().storage, &Uint128::from(2u8), 25)?;
    snapshot_item.save(deps.as_mut().storage, &Uint128::from(3u8), 30)?;
    snapshot_item.save(deps.as_mut().storage, &Uint128::from(4u8), 35)?;
    snapshot_item.save(deps.as_mut().storage, &Uint128::from(5u8), 40)?;

    let checkpoints = get_checkpoints(deps.as_ref(), snapshot_item)?;

    assert_eq!(
        checkpoints,
        vec![
            TotalWeightCheckpoint { height: 20, total_weight: Uint128::from(1u8) },
            TotalWeightCheckpoint { height: 25, total_weight: Uint128::from(2u8) },
            TotalWeightCheckpoint { height: 30, total_weight: Uint128::from(3u8) },
            TotalWeightCheckpoint { height: 35, total_weight: Uint128::from(4u8) },
            TotalWeightCheckpoint { height: 40, total_weight: Uint128::from(5u8) },
        ]
    );

    Ok(())
}