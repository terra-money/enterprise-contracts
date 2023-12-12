use std::{env::current_dir, fs::create_dir_all};

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use enterprise_factory_api::api::{
    AllDaosResponse, ConfigResponse, EnterpriseCodeIdsResponse, IsEnterpriseCodeIdResponse,
};
use enterprise_factory_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(AllDaosResponse), &out_dir);
    export_schema(&schema_for!(EnterpriseCodeIdsResponse), &out_dir);
    export_schema(&schema_for!(IsEnterpriseCodeIdResponse), &out_dir);
}
