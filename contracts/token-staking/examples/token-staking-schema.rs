use std::{env::current_dir, fs::create_dir_all};

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use staking_common::api::TotalStakedAmountResponse;
use staking_common::msg::InstantiateMsg;
use token_staking_api::api::{ClaimsResponse, UserTokenStakeResponse};
use token_staking_api::msg::{ExecuteMsg, MigrateMsg, QueryMsg};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);
    export_schema(&schema_for!(UserTokenStakeResponse), &out_dir);
    export_schema(&schema_for!(TotalStakedAmountResponse), &out_dir);
    export_schema(&schema_for!(ClaimsResponse), &out_dir);
}
