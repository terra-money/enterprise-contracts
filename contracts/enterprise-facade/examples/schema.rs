use std::{env::current_dir, fs::create_dir_all};

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use enterprise_facade_api::api::{
    AdapterResponse, AssetWhitelistResponse, ClaimsResponse, DaoInfoResponse, MemberInfoResponse,
    MemberVoteResponse, MultisigMembersResponse, NftWhitelistResponse, ProposalResponse,
    ProposalStatusResponse, ProposalVotesResponse, ProposalsResponse, StakedNftsResponse,
    TotalStakedAmountResponse, UserStakeResponse,
};
use enterprise_facade_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(AdapterResponse), &out_dir);
    export_schema(&schema_for!(StakedNftsResponse), &out_dir);
    export_schema(&schema_for!(MultisigMembersResponse), &out_dir);
    export_schema(&schema_for!(DaoInfoResponse), &out_dir);
    export_schema(&schema_for!(AssetWhitelistResponse), &out_dir);
    export_schema(&schema_for!(NftWhitelistResponse), &out_dir);
    export_schema(&schema_for!(MemberInfoResponse), &out_dir);
    export_schema(&schema_for!(ProposalResponse), &out_dir);
    export_schema(&schema_for!(ProposalsResponse), &out_dir);
    export_schema(&schema_for!(ProposalStatusResponse), &out_dir);
    export_schema(&schema_for!(MemberVoteResponse), &out_dir);
    export_schema(&schema_for!(ProposalVotesResponse), &out_dir);
    export_schema(&schema_for!(UserStakeResponse), &out_dir);
    export_schema(&schema_for!(TotalStakedAmountResponse), &out_dir);
    export_schema(&schema_for!(StakedNftsResponse), &out_dir);
    export_schema(&schema_for!(ClaimsResponse), &out_dir);
}
