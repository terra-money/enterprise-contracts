use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

pub const MIGRATION_TO_V_1_0_0_STAGE: Item<MigrationStage> = Item::new("migration_stage");

#[cw_serde]
pub enum MigrationStage {
    /// Initial state
    MigrationNotStarted,
    /// Stage where we migrated everything except large items (stakes and claims).
    InitialMigrationFinished,
    /// Stage where stakes and claims are being migrated, which possibly requires multiple steps to
    /// resolve.
    MigrateAssets,
    /// Migration of everything has been completed.
    Finalized,
}
