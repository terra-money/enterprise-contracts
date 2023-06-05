use crate::api::{
    ClaimMsg, ClaimsParams, ClaimsResponse, ReceiveNftMsg, TotalStakedAmountParams,
    TotalStakedAmountResponse, UnstakeMsg, UpdateConfigMsg, UserNftStakeParams,
    UserNftStakeResponse, UserNftTotalStakeParams, UserNftTotalStakeResponse,
};
use common::cw::ReleaseAt;
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
    AddClaim { user: String, release_at: ReleaseAt },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(UserNftStakeResponse)]
    UserStake(UserNftStakeParams),
    #[returns(UserNftTotalStakeResponse)]
    UserTotalStake(UserNftTotalStakeParams),
    #[returns(TotalStakedAmountResponse)]
    TotalStakedAmount(TotalStakedAmountParams),
    #[returns(ClaimsResponse)]
    Claims(ClaimsParams),
    #[returns(ClaimsResponse)]
    ReleasableClaims(ClaimsParams),
}

#[cw_serde]
pub struct MigrateMsg {}
