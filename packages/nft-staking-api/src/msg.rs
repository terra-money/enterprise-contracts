use crate::api::{
    ClaimMsg, ClaimsParams, ClaimsResponse, ReceiveNftMsg, TotalStakedAmountResponse, UnstakeMsg,
    UpdateAdminMsg, UserNftStakeParams, UserNftStakeResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub nft_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Unstake(UnstakeMsg),
    Claim(ClaimMsg),
    UpdateAdmin(UpdateAdminMsg),
    ReceiveNft(ReceiveNftMsg),
}

#[cw_serde]
pub enum Cw721HookMsg {
    Stake {},
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
