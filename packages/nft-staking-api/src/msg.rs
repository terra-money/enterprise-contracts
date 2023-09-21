use crate::api::{
    ClaimMsg, ClaimsParams, ClaimsResponse, NftConfigResponse, ReceiveNftMsg, StakedNftsParams,
    StakedNftsResponse, UnstakeMsg, UpdateUnlockingPeriodMsg, UserNftStakeParams,
    UserNftStakeResponse,
};
use common::cw::ReleaseAt;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_utils::Duration;
use membership_common_api::api::{
    MembersParams, MembersResponse, TotalWeightParams, TotalWeightResponse, UserWeightParams,
    UserWeightResponse, WeightChangeHookMsg,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_contract: String,
    pub nft_contract: String,
    pub unlocking_period: Duration,
    pub weight_change_hooks: Option<Vec<String>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Unstake(UnstakeMsg),
    Claim(ClaimMsg),
    UpdateUnlockingPeriod(UpdateUnlockingPeriodMsg),
    ReceiveNft(ReceiveNftMsg),
    AddWeightChangeHook(WeightChangeHookMsg),
    RemoveWeightChangeHook(WeightChangeHookMsg),
}

#[cw_serde]
pub enum Cw721HookMsg {
    Stake { user: String },
    AddClaim { user: String, release_at: ReleaseAt },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(NftConfigResponse)]
    NftConfig {},
    #[returns(UserNftStakeResponse)]
    UserStake(UserNftStakeParams),
    #[returns(UserWeightResponse)]
    UserWeight(UserWeightParams),
    #[returns(StakedNftsResponse)]
    StakedNfts(StakedNftsParams),
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
