use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct Version {
    pub version: u64,
    /// Changelog items from the previous version
    pub changelog: Vec<String>,
    pub enterprise_controller_code_id: u64,
    pub token_staking_membership_code_id: u64,
    pub nft_staking_membership_code_id: u64,
    pub multisig_staking_membership_code_id: u64,
    pub funds_distributor_code_id: u64,
    pub enterprise_governance_code_id: u64,
}

#[cw_serde]
pub struct AddVersionMsg {
    pub version: Version,
}

#[cw_serde]
pub struct VersionParams {
    pub version: u64,
}

#[cw_serde]
pub struct VersionsParams {
    pub start_after: Option<u64>,
    pub limit: Option<u32>,
}

////// Responses

#[cw_serde]
pub struct AdminResponse {
    pub admin: Addr,
}

#[cw_serde]
pub struct VersionResponse {
    pub version: Version,
}

#[cw_serde]
pub struct VersionsResponse {
    pub versions: Vec<Version>,
}
