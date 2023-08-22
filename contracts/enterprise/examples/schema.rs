use std::{env::current_dir, fs::create_dir_all};

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use enterprise_protocol::api::{
    ComponentContractsResponse, CrossChainTreasuriesResponse, CrossChainTreasuryParams,
    DaoInfoResponse,
};

use enterprise_protocol::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);
    export_schema(&schema_for!(DaoInfoResponse), &out_dir);
    export_schema(&schema_for!(ComponentContractsResponse), &out_dir);
    export_schema(&schema_for!(CrossChainTreasuriesResponse), &out_dir);
    export_schema(&schema_for!(CrossChainTreasuryParams), &out_dir);
}
