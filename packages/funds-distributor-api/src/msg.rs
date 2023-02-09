use crate::api::{
    ClaimRewardsMsg, UpdateTotalStakedMsg, UpdateUserStakeMsg, UserRewardsParams,
    UserRewardsResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateTotalStaked(UpdateTotalStakedMsg),
    UpdateUserStake(UpdateUserStakeMsg),
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
