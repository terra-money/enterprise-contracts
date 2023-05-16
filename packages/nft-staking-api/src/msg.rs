use crate::api::{
    ClaimsResponse, UnstakeMsg, UserNftStakeParams, UserNftStakeResponse, UserNftTotalStakeResponse,
};
use common::cw::ReleaseAt;
use common::nft::ReceiveNftMsg;
use cosmwasm_schema::{cw_serde, QueryResponses};
use staking_common::api::{
    ClaimMsg, ClaimsParams, TotalStakedAmountParams, TotalStakedAmountResponse, UpdateConfigMsg,
    UserTotalStakeParams,
};

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
    UserTotalStake(UserTotalStakeParams),
    #[returns(TotalStakedAmountResponse)]
    TotalStakedAmount(TotalStakedAmountParams),
    #[returns(ClaimsResponse)]
    Claims(ClaimsParams),
    #[returns(ClaimsResponse)]
    ReleasableClaims(ClaimsParams),
}

#[cw_serde]
pub struct MigrateMsg {}
