use common::commons::ModifyValue;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Timestamp};
use enterprise_versioning_api::api::Version;
use std::fmt;
use strum_macros::Display;

#[cw_serde]
#[derive(Display)]
pub enum DaoType {
    Denom,
    Token,
    Nft,
    Multisig,
}

#[cw_serde]
pub struct DaoMetadata {
    pub name: String,
    pub description: Option<String>,
    pub logo: Logo,
    pub socials: DaoSocialData,
}

#[cw_serde]
pub enum Logo {
    // TODO: think about allowing on-chain logo
    Url(String),
    None,
}

impl fmt::Display for Logo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Logo::Url(url) => write!(f, "url: {}", url),
            Logo::None => write!(f, "none"),
        }
    }
}

#[cw_serde]
pub struct DaoSocialData {
    pub github_username: Option<String>,
    pub discord_username: Option<String>,
    pub twitter_username: Option<String>,
    pub telegram_username: Option<String>,
}

#[cw_serde]
pub struct FinalizeInstantiationMsg {
    pub enterprise_treasury_contract: String,
    pub enterprise_governance_contract: String,
    pub enterprise_governance_controller_contract: String,
    pub enterprise_outposts_contract: String,
    pub funds_distributor_contract: String,
    pub membership_contract: String,
    pub council_membership_contract: String,
    pub attestation_contract: Option<String>,
}

#[cw_serde]
pub struct UpdateMetadataMsg {
    pub name: ModifyValue<String>,
    pub description: ModifyValue<Option<String>>,
    pub logo: ModifyValue<Logo>,
    pub github_username: ModifyValue<Option<String>>,
    pub discord_username: ModifyValue<Option<String>>,
    pub twitter_username: ModifyValue<Option<String>>,
    pub telegram_username: ModifyValue<Option<String>>,
}

/// MigrateMsg for a specific version.
#[cw_serde]
pub struct VersionMigrateMsg {
    pub version: Version,
    pub migrate_msg: Binary,
}

#[cw_serde]
pub struct UpgradeDaoMsg {
    pub new_version: Version,
    /// Expects an array of (version, migrate msg for that version).
    /// E.g.
    /// [
    ///   {
    ///     "version": {
    ///       "major": 2,
    ///       "minor": 0,
    ///       "patch": 0
    ///     },
    ///     "migrate_msg": <MigrateMsg JSON for 2.0.0>
    ///   },
    ///   {
    ///     "version": {
    ///       "major": 2,
    ///       "minor": 1,
    ///       "patch": 3
    ///     },
    ///     "migrate_msg": <MigrateMsg JSON for 2.1.3>
    ///   }
    /// ]
    pub migrate_msgs: Vec<VersionMigrateMsg>,
}

#[cw_serde]
pub struct SetAttestationMsg {
    pub attestation_text: String,
}

#[cw_serde]
pub struct ExecuteMsgsMsg {
    pub msgs: Vec<String>,
}

#[cw_serde]
pub struct UpdateConfigMsg {
    pub new_versioning_contract: ModifyValue<String>,
    pub new_factory_contract: ModifyValue<String>,
}

#[cw_serde]
pub struct IsRestrictedUserParams {
    pub user: String,
}

// Responses

#[cw_serde]
pub struct DaoInfoResponse {
    pub creation_date: Timestamp,
    pub metadata: DaoMetadata,
    pub dao_type: DaoType,
    pub dao_version: Version,
}

#[cw_serde]
pub struct ComponentContractsResponse {
    pub enterprise_factory_contract: Addr,
    pub enterprise_versioning_contract: Addr,
    pub enterprise_governance_contract: Addr,
    pub enterprise_governance_controller_contract: Addr,
    pub enterprise_outposts_contract: Addr,
    pub enterprise_treasury_contract: Addr,
    pub funds_distributor_contract: Addr,
    pub membership_contract: Addr,
    pub council_membership_contract: Addr,
    pub attestation_contract: Option<Addr>,
}

#[cw_serde]
pub struct IsRestrictedUserResponse {
    pub is_restricted: bool,
}
