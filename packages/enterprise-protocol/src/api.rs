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
    pub funds_distributor_contract: String,
    pub membership_contract: String,
    pub council_membership_contract: String,
    pub attestation_contract: Option<String>,
    pub dao_type: DaoType,
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

#[cw_serde]
pub struct UpgradeDaoMsg {
    pub new_version: Version,
    /// Expects a map of (version, migrate msg for that version).
    /// E.g.
    /// {
    ///   "1.0.2": { <MigrateMsg for 1.0.2> },
    ///   "2.0.0": { <MigrateMsg for 2.0.0> }
    /// }
    pub migrate_msg: Binary,
}

#[cw_serde]
pub struct SetAttestationMsg {
    pub attestation_text: String,
}

#[cw_serde]
pub struct AddCrossChainProxyMsg {
    pub chain_id: String,
    pub proxy_addr: String,
}

#[cw_serde]
pub struct AddCrossChainTreasuryMsg {
    pub chain_id: String,
    pub treasury_addr: String,
}

#[cw_serde]
pub struct IsRestrictedUserParams {
    pub user: String,
}

#[cw_serde]
pub struct CrossChainTreasury {
    pub chain_id: String,
    pub treasury_addr: String,
}

#[cw_serde]
pub struct CrossChainTreasuriesParams {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct CrossChainDeploymentsParams {
    pub chain_id: String,
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
    pub enterprise_governance_contract: Addr,
    pub enterprise_governance_controller_contract: Addr,
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

#[cw_serde]
pub struct CrossChainTreasuriesResponse {
    pub treasuries: Vec<CrossChainTreasury>,
}

#[cw_serde]
pub struct CrossChainDeploymentsResponse {
    pub chain_id: String,
    pub proxy_addr: Option<String>,
    pub treasury_addr: Option<String>,
}
