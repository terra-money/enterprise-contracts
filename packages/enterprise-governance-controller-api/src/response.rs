use crate::api::{ProposalId, ProposalType};
use cosmwasm_std::{Response, Uint128};
use poll_engine_api::api::{PollId, VoteOutcome};

pub fn instantiate_response() -> Response {
    Response::new().add_attribute("action", "instantiate")
}

pub fn execute_create_proposal_response(dao_address: String) -> Response {
    Response::new()
        .add_attribute("action", "create_proposal")
        .add_attribute("dao_address", dao_address)
}

pub fn reply_create_poll_response(poll_id: PollId) -> Response {
    Response::new().add_attribute("proposal_id", poll_id.to_string())
}

pub fn execute_create_council_proposal_response(dao_address: String) -> Response {
    Response::new()
        .add_attribute("action", "create_council_proposal")
        .add_attribute("dao_address", dao_address)
}

pub fn execute_cast_vote_response(
    dao_address: String,
    proposal_id: ProposalId,
    voter: String,
    outcome: VoteOutcome,
    amount: Uint128,
) -> Response {
    Response::new()
        .add_attribute("action", "cast_vote")
        .add_attribute("dao_address", dao_address)
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("voter", voter)
        .add_attribute("outcome", outcome.to_string())
        .add_attribute("amount", amount.to_string())
}

pub fn execute_cast_council_vote_response(
    dao_address: String,
    proposal_id: ProposalId,
    voter: String,
    outcome: VoteOutcome,
    amount: Uint128,
) -> Response {
    Response::new()
        .add_attribute("action", "cast_council_vote")
        .add_attribute("dao_address", dao_address)
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("voter", voter)
        .add_attribute("outcome", outcome.to_string())
        .add_attribute("amount", amount.to_string())
}

pub fn execute_execute_proposal_response(
    dao_address: String,
    proposal_id: ProposalId,
    proposal_type: ProposalType,
) -> Response {
    Response::new()
        .add_attribute("action", "execute_proposal")
        .add_attribute("dao_address", dao_address)
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("proposal_type", proposal_type.to_string())
}

pub fn execute_weights_changed_response() -> Response {
    Response::new().add_attribute("action", "weights_changed")
}

pub fn execute_execute_msg_reply_callback_response() -> Response {
    Response::new().add_attribute("action", "execute_msg_reply_callback")
}
