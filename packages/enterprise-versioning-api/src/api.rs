use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError};
use std::fmt;
use std::str::FromStr;

// TODO: tests for this for comparison and parsing?
#[cw_serde]
#[derive(Ord, PartialOrd, Eq, Hash)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl Version {
    pub fn new(major: u64, minor: u64, patch: u64) -> Version {
        Version {
            major,
            minor,
            patch,
        }
    }
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

impl FromStr for Version {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();

        if parts.len() != 3 {
            return Err(StdError::generic_err(
                "version string must have exactly two dots",
            ));
        }

        let major = parts[0]
            .parse::<u64>()
            .map_err(|_| StdError::generic_err("major version is not a valid number"))?;
        let minor = parts[1]
            .parse::<u64>()
            .map_err(|_| StdError::generic_err("minor version is not a valid number"))?;
        let patch = parts[2]
            .parse::<u64>()
            .map_err(|_| StdError::generic_err("patch version is not a valid number"))?;

        Ok(Version {
            major,
            minor,
            patch,
        })
    }
}

#[cw_serde]
pub struct VersionInfo {
    pub version: Version,
    /// Changelog items from the previous version
    pub changelog: Vec<String>,
    pub attestation_code_id: u64,
    pub enterprise_code_id: u64,
    pub enterprise_governance_code_id: u64,
    pub enterprise_governance_controller_code_id: u64,
    pub enterprise_outposts_code_id: u64,
    pub enterprise_treasury_code_id: u64,
    pub funds_distributor_code_id: u64,
    pub token_staking_membership_code_id: u64,
    pub denom_staking_membership_code_id: u64,
    pub nft_staking_membership_code_id: u64,
    pub multisig_membership_code_id: u64,
}

#[cw_serde]
pub struct UpdateAdminMsg {
    pub new_admin: String,
}

#[cw_serde]
pub struct AddVersionMsg {
    pub version: VersionInfo,
}

#[cw_serde]
pub struct EditVersionMsg {
    pub version: Version,
    pub changelog: Option<Vec<String>>,
    pub attestation_code_id: Option<u64>,
    pub enterprise_code_id: Option<u64>,
    pub enterprise_governance_code_id: Option<u64>,
    pub enterprise_governance_controller_code_id: Option<u64>,
    pub enterprise_outposts_code_id: Option<u64>,
    pub enterprise_treasury_code_id: Option<u64>,
    pub funds_distributor_code_id: Option<u64>,
    pub token_staking_membership_code_id: Option<u64>,
    pub denom_staking_membership_code_id: Option<u64>,
    pub nft_staking_membership_code_id: Option<u64>,
    pub multisig_membership_code_id: Option<u64>,
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
