use crate::api::{PollStatus, PollType, VotingScheme};
use cosmwasm_std::{Addr, Decimal, Storage, Timestamp};

use crate::state::{GovStateExt, Poll, GOV_STATE};

pub fn mock_poll(store: &mut dyn Storage) -> Poll {
    mock_poll_with_id(GOV_STATE.increment_poll_id(store).unwrap())
}

pub fn mock_poll_with_id(id: u64) -> Poll {
    Poll {
        id,
        proposer: Addr::unchecked("proposer"),
        deposit_amount: 1_000_000,
        label: "some label".to_string(),
        description: "some description".to_string(),
        poll_type: PollType::Multichoice {
            threshold: Decimal::percent(50),
            n_outcomes: 3,
            rejecting_outcomes: vec![1],
            abstaining_outcomes: vec![2],
        },
        scheme: VotingScheme::CoinVoting,
        status: PollStatus::InProgress {
            ends_at: Timestamp::from_nanos(3),
        },
        started_at: Default::default(),
        ends_at: Timestamp::from_nanos(3),
        quorum: Default::default(),
        results: Default::default(),
    }
}
