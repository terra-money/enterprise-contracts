use crate::api::{
    AddVersionMsg, AdminResponse, VersionParams, VersionResponse, VersionsParams, VersionsResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddVersion(AddVersionMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AdminResponse)]
    Admin {},
    #[returns(VersionResponse)]
    Version(VersionParams),
    #[returns(VersionsResponse)]
    Versions(VersionsParams),
    #[returns(VersionResponse)]
    LatestVersion {},
}

#[cw_serde]
pub struct MigrateMsg {}
