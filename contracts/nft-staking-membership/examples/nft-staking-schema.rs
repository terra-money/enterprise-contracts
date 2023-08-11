use std::{env::current_dir, fs::create_dir_all};

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use membership_common_api::api::{
    AdminResponse, MembersResponse, TotalWeightResponse, UserWeightResponse,
};
use nft_staking_api::api::{ClaimsResponse, NftConfigResponse, UserNftStakeResponse};
use nft_staking_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);
    export_schema(&schema_for!(TotalWeightResponse), &out_dir);
    export_schema(&schema_for!(AdminResponse), &out_dir);
    export_schema(&schema_for!(MembersResponse), &out_dir);
    export_schema(&schema_for!(TotalWeightResponse), &out_dir);
    export_schema(&schema_for!(UserWeightResponse), &out_dir);
    export_schema(&schema_for!(ClaimsResponse), &out_dir);
    export_schema(&schema_for!(NftConfigResponse), &out_dir);
    export_schema(&schema_for!(UserNftStakeResponse), &out_dir);
}
