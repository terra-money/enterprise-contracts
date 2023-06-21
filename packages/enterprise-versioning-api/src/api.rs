use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use std::fmt;

#[cw_serde]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl From<Version> for (u64, u64, u64) {
    fn from(version: Version) -> Self {
        (version.major, version.minor, version.patch)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cw_serde]
pub struct VersionInfo {
    pub version: Version,
    /// Changelog items from the previous version
    pub changelog: Vec<String>,
    pub enterprise_code_id: u64,
    pub enterprise_governance_code_id: u64,
    pub enterprise_governance_controller_code_id: u64,
    pub enterprise_treasury_code_id: u64,
    pub funds_distributor_code_id: u64,
    pub token_staking_membership_code_id: u64,
    pub nft_staking_membership_code_id: u64,
    pub multisig_membership_code_id: u64,
}

#[cw_serde]
pub struct AddVersionMsg {
    pub version: VersionInfo,
}

#[cw_serde]
pub struct VersionParams {
    pub version: Version,
}

#[cw_serde]
pub struct VersionsParams {
    pub start_after: Option<Version>,
    pub limit: Option<u32>,
}

////// Responses

#[cw_serde]
pub struct AdminResponse {
    pub admin: Addr,
}

#[cw_serde]
pub struct VersionResponse {
    pub version: VersionInfo,
}

#[cw_serde]
pub struct VersionsResponse {
    pub versions: Vec<VersionInfo>,
}
