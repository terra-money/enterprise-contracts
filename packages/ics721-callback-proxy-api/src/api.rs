use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct ConfigResponse {
    pub ics721_proxy: Addr,
}
