use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Cw3ListVoters {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct Cw3VoterListResponse {
    pub voters: Vec<Cw3VoterDetail>,
}

#[cw_serde]
pub struct Cw3VoterDetail {
    pub addr: String,
    pub weight: u64,
}
