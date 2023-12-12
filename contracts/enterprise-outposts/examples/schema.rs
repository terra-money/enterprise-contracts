use std::{env::current_dir, fs::create_dir_all};

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use enterprise_outposts_api::api::{CrossChainTreasuriesParams, CrossChainTreasuriesResponse};
use enterprise_outposts_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);
    export_schema(&schema_for!(CrossChainTreasuriesResponse), &out_dir);
    export_schema(&schema_for!(CrossChainTreasuriesParams), &out_dir);
}
