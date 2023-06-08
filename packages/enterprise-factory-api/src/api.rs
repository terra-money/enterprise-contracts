use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128, Uint64};
use cw_asset::AssetInfo;
use enterprise_governance_controller_api::api::{GovConfig, ProposalActionType};
use enterprise_protocol::api::{DaoMetadata, ExistingDaoMembershipMsg, NewMembershipInfo};

#[cw_serde]
pub struct Config {
    pub enterprise_code_id: u64,
    pub enterprise_governance_code_id: u64,
    pub funds_distributor_code_id: u64,
    pub cw3_fixed_multisig_code_id: u64,
    pub cw20_code_id: u64,
    pub cw721_code_id: u64,
}

#[cw_serde]
pub struct DaoCouncilSpec {
    /// Addresses of council members. Each member has equal voting power.
    pub members: Vec<String>,
    /// Portion of total available votes cast in a proposal to consider it valid
    /// e.g. quorum of 30% means that 30% of all available votes have to be cast in the proposal,
    /// otherwise it fails automatically when it expires
    pub quorum: Decimal,
    /// Portion of votes assigned to a single option from all the votes cast in the given proposal
    /// required to determine the 'winning' option
    /// e.g. 51% threshold means that an option has to have at least 51% of the cast votes to win
    pub threshold: Decimal,
    /// Proposal action types allowed in proposals that are voted on by the council.
    /// Effectively defines what types of actions council can propose and vote on.
    /// If None, will default to a predefined set of actions.
    pub allowed_proposal_action_types: Option<Vec<ProposalActionType>>,
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct CreateDaoMsg {
    pub dao_metadata: DaoMetadata,
    pub dao_gov_config: GovConfig,
    /// Optional council structure that can manage certain aspects of the DAO
    pub dao_council: Option<DaoCouncilSpec>,
    pub dao_membership: CreateDaoMembershipMsg,
    /// assets that are allowed to show in DAO's treasury
    pub asset_whitelist: Option<Vec<AssetInfo>>,
    /// NFTs that are allowed to show in DAO's treasury
    pub nft_whitelist: Option<Vec<Addr>>,
    /// Minimum weight that a user should have in order to qualify for rewards.
    /// E.g. a value of 3 here means that a user in token or NFT DAO needs at least 3 staked
    /// DAO assets, or a weight of 3 in multisig DAO, to be eligible for rewards.
    pub minimum_weight_for_rewards: Option<Uint128>,
}

#[cw_serde]
#[allow(clippy::large_enum_variant)]
pub enum CreateDaoMembershipMsg {
    NewMembership(NewMembershipInfo),
    ExistingMembership(ExistingDaoMembershipMsg),
}

#[cw_serde]
pub struct QueryAllDaosMsg {
    pub start_after: Option<Uint64>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct AllDaosResponse {
    pub daos: Vec<Addr>,
}

#[cw_serde]
pub struct EnterpriseCodeIdsMsg {
    pub start_after: Option<Uint64>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct EnterpriseCodeIdsResponse {
    pub code_ids: Vec<Uint64>,
}

#[cw_serde]
pub struct IsEnterpriseCodeIdMsg {
    pub code_id: Uint64,
}

#[cw_serde]
pub struct IsEnterpriseCodeIdResponse {
    pub is_enterprise_code_id: bool,
}
