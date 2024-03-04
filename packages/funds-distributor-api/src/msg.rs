use crate::api::{
    ClaimRewardsMsg, DistributionType, MinimumEligibleWeightResponse,
    NumberProposalsTrackedResponse, ProposalIdsTrackedResponse, UpdateMinimumEligibleWeightMsg,
    UpdateUserWeightsMsg, UserRewardsParams, UserRewardsResponse, UserWeight,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub enterprise_contract: String,
    pub initial_weights: Vec<UserWeight>,
    /// Optional minimum weight that the user must have to be eligible for rewards distributions
    pub minimum_eligible_weight: Option<Uint128>,
    /// Number of last proposals to track for user participation. If not set, will result to 0
    pub participation_proposals_tracked: Option<u8>,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateUserWeights(UpdateUserWeightsMsg),
    UpdateMinimumEligibleWeight(UpdateMinimumEligibleWeightMsg),
    DistributeNative {
        distribution_type: Option<DistributionType>,
    },
    ClaimRewards(ClaimRewardsMsg),
    Receive(Cw20ReceiveMsg),
}

#[cw_serde]
pub enum Cw20HookMsg {
    Distribute {
        distribution_type: Option<DistributionType>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(UserRewardsResponse)]
    UserRewards(UserRewardsParams),
    #[returns(MinimumEligibleWeightResponse)]
    MinimumEligibleWeight {},
    #[returns(NumberProposalsTrackedResponse)]
    NumberProposalsTracked {},
    #[returns(ProposalIdsTrackedResponse)]
    ProposalIdsTracked {},
}

#[cw_serde]
pub struct MigrateMsg {}
