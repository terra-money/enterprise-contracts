use crate::api::{
    ClaimRewardsMsg, UpdateUserWeightsMsg, UserRewardsParams, UserRewardsResponse, UserWeight,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_contract: String,
    pub initial_weights: Vec<UserWeight>,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateUserWeights(UpdateUserWeightsMsg),
    DistributeNative {},
    ClaimRewards(ClaimRewardsMsg),
    Receive(Cw20ReceiveMsg),
}

#[cw_serde]
pub enum Cw20HookMsg {
    Distribute {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(UserRewardsResponse)]
    UserRewards(UserRewardsParams),
}

#[cw_serde]
pub struct MigrateMsg {}
