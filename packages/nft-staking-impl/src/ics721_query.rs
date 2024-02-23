use crate::ics721_query::Ics721QueryMsg::NftContract;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, StdResult};

pub fn query_ics721_proxy_nft_addr(
    deps: Deps,
    ics721_proxy_addr: String,
    class_id: String,
) -> StdResult<Option<Addr>> {
    let response: Option<String> = deps
        .querier
        .query_wasm_smart(ics721_proxy_addr, &NftContract { class_id })?;

    response
        .map(|addr| deps.api.addr_validate(&addr))
        .transpose()
}

#[cw_serde]
enum Ics721QueryMsg {
    /// Gets the NFT contract associated wtih the provided class
    /// ID. If no such contract exists, returns None. Returns
    /// Option<Addr>.
    NftContract { class_id: String },
}
