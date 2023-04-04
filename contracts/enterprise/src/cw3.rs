use cosmwasm_schema::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Cw3ListVoters {
    pub start_after: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct Cw3VoterListResponse {
    pub voters: Vec<Cw3VoterDetail>,
}

#[derive(Serialize, Deserialize)]
pub struct Cw3VoterDetail {
    pub addr: String,
    pub weight: u64,
}
