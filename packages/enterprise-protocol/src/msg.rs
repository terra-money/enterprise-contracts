use crate::api::{
    CreateProposalMsg, DaoCouncilSpec, DaoInfoResponse, DaoMembershipInfo, DaoMetadata,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_governance_code_id: u64,
    pub funds_distributor_code_id: u64,
    pub dao_metadata: DaoMetadata,
    /// Optional council structure that can manage certain aspects of the DAO
    pub dao_council: Option<DaoCouncilSpec>,
    pub dao_membership_info: DaoMembershipInfo,
    /// Address of enterprise-factory contract that is creating this DAO
    pub enterprise_factory_contract: String,
    /// Minimum weight that a user should have in order to qualify for rewards.
    /// E.g. a value of 3 here means that a user in token or NFT DAO needs at least 3 staked
    /// DAO assets, or a weight of 3 in multisig DAO, to be eligible for rewards.
    pub minimum_weight_for_rewards: Option<Uint128>,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
pub enum Cw20HookMsg {
    Stake {},
    CreateProposal(CreateProposalMsg),
}

#[cw_serde]
pub enum Cw721HookMsg {
    Stake {},
}

#[cw_serde]
pub struct MigrateMsg {
    pub minimum_eligible_weight: Option<Uint128>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DaoInfoResponse)]
    DaoInfo {},
}
