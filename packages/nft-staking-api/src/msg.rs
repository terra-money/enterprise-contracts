use crate::api::{
    ClaimMsg, ClaimsParams, ClaimsResponse, ReceiveNftMsg, TotalStakedAmountResponse, UnstakeMsg,
    UpdateConfigMsg, UserNftStakeParams, UserNftStakeResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_utils::Duration;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub nft_contract: String,
    pub unlocking_period: Duration,
}

#[cw_serde]
pub enum ExecuteMsg {
    Unstake(UnstakeMsg),
    Claim(ClaimMsg),
    UpdateConfig(UpdateConfigMsg),
    ReceiveNft(ReceiveNftMsg),
}

#[cw_serde]
pub enum Cw721HookMsg {
    Stake { user: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(UserNftStakeResponse)]
    UserStake(UserNftStakeParams),
    #[returns(TotalStakedAmountResponse)]
    TotalStakedAmount {},
    #[returns(ClaimsResponse)]
    Claims(ClaimsParams),
}

#[cw_serde]
pub struct MigrateMsg {}
