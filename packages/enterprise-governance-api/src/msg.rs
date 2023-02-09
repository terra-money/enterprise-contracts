use cosmwasm_schema::{cw_serde, QueryResponses};
use poll_engine_api::api::{
    CastVoteParams, CreatePollParams, EndPollParams, PollId, PollParams, PollResponse,
    PollStatusResponse, PollVoterParams, PollVoterResponse, PollVotersParams, PollVotersResponse,
    PollsParams, PollsResponse, UpdateVotesParams, VoterParams, VoterResponse,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub enterprise_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreatePoll(CreatePollParams),
    CastVote(CastVoteParams),
    UpdateVotes(UpdateVotesParams),
    EndPoll(EndPollParams),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(PollResponse)]
    Poll(PollParams),
    #[returns(PollsResponse)]
    Polls(PollsParams),
    #[returns(PollStatusResponse)]
    PollStatus { poll_id: PollId },
    #[returns(PollVoterResponse)]
    PollVoter(PollVoterParams),
    #[returns(PollVotersResponse)]
    PollVoters(PollVotersParams),
    #[returns(VoterResponse)]
    Voter(VoterParams),
}

#[cw_serde]
pub struct MigrateMsg {}
