use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

pub const MIGRATION_TO_V_1_0_0_STAGE: Item<MigrationStage> = Item::new("migration_stage");

#[cw_serde]
pub enum MigrationStage {
    /// Initial state
    MigrationNotStarted,
    /// Stage where we began migration, but there are still elements to be migrated.
    MigrationInProgress,
    /// Migration of everything has been completed.
    MigrationCompleted,
}
