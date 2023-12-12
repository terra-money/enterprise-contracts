use std::{env::current_dir, fs::create_dir_all};

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use enterprise_governance_controller_api::api::{
    GovConfigResponse, MemberVoteResponse, ProposalResponse, ProposalStatusResponse,
    ProposalVotesResponse, ProposalsResponse,
};
use enterprise_governance_controller_api::msg::{
    Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(Cw20HookMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);
    export_schema(&schema_for!(GovConfigResponse), &out_dir);
    export_schema(&schema_for!(ProposalStatusResponse), &out_dir);
    export_schema(&schema_for!(ProposalResponse), &out_dir);
    export_schema(&schema_for!(ProposalsResponse), &out_dir);
    export_schema(&schema_for!(MemberVoteResponse), &out_dir);
    export_schema(&schema_for!(ProposalVotesResponse), &out_dir);
}
