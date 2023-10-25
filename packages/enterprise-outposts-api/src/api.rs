use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct AddCrossChainProxyMsg {
    pub chain_id: String,
    pub proxy_addr: String,
}

#[cw_serde]
pub struct AddCrossChainTreasuryMsg {
    pub chain_id: String,
    pub treasury_addr: String,
}

#[cw_serde]
pub struct CrossChainTreasury {
    pub chain_id: String,
    pub treasury_addr: String,
}

#[cw_serde]
pub struct CrossChainTreasuriesParams {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct CrossChainDeploymentsParams {
    pub chain_id: String,
}

// Responses

#[cw_serde]
pub struct CrossChainTreasuriesResponse {
    pub treasuries: Vec<CrossChainTreasury>,
}

#[cw_serde]
pub struct CrossChainDeploymentsResponse {
    pub chain_id: String,
    pub proxy_addr: Option<String>,
    pub treasury_addr: Option<String>,
}
