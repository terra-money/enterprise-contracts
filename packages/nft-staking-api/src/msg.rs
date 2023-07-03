use crate::api::{
    ClaimMsg, ClaimsParams, ClaimsResponse, ConfigResponse, ReceiveNftMsg, UnstakeMsg,
    UpdateConfigMsg, UserNftStakeParams, UserNftStakeResponse,
};
use common::cw::ReleaseAt;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_utils::Duration;
use membership_common::api::{
    MembersParams, MembersResponse, TotalWeightParams, TotalWeightResponse, UserWeightParams,
    UserWeightResponse,
};

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
    #[returns(ConfigResponse)]
    Config {},
    #[returns(UserNftStakeResponse)]
    UserStake(UserNftStakeParams),
    #[returns(UserWeightResponse)]
    UserWeight(UserWeightParams),
    #[returns(TotalWeightResponse)]
    TotalWeight(TotalWeightParams),
    #[returns(ClaimsResponse)]
    Claims(ClaimsParams),
    #[returns(ClaimsResponse)]
    ReleasableClaims(ClaimsParams),
    #[returns(MembersResponse)]
    Members(MembersParams),
}

#[cw_serde]
pub struct MigrateMsg {}
