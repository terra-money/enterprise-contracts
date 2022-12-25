use std::collections::BTreeMap;

use cosmwasm_std::{to_binary, Addr, Decimal, DepsMut, Env, Storage, Timestamp, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
use itertools::Itertools;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use common::cw::RangeArgs;

use crate::api::VoteOutcome::{Abstain, No, Veto};
use crate::api::{
    CreatePollParams, PollId, PollRejectionReason, PollStatus, PollStatusFilter, Vote, VoteOutcome,
    VotingScheme,
};
use crate::error::*;

pub const GOV_STATE: Item<GovState> = Item::new("gov_state");

#[serde_as]
#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct GovState {
    pub poll_count: u64,
}

pub trait GovStateExt {
    fn increment_poll_id(&self, store: &mut dyn Storage) -> PollResult<PollId>;
}

impl GovStateExt for Item<'_, GovState> {
    fn increment_poll_id(&self, store: &mut dyn Storage) -> PollResult<PollId> {
        let mut state = self.load(store)?;
        state.poll_count += 1;
        self.save(store, &state)?;
        Ok(state.poll_count)
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// A poll.
pub struct Poll {
    /// Unique identifier for the poll.
    pub id: PollId,
    /// Proposer address.
    pub proposer: Addr,
    /// Poll deposit amount.
    pub deposit_amount: u128,
    /// User-defined label for the poll.
    pub label: String,
    /// User-defined description for the poll.
    pub description: String,
    /// Voting scheme of the poll, e.g. "CoinVoting".
    pub scheme: VotingScheme,
    /// Status of the poll.
    pub status: PollStatus,
    /// Start-time of poll.
    pub started_at: Timestamp,
    /// End-time of poll.
    pub ends_at: Timestamp,
    /// Quorum to be reached for the poll to be valid.
    /// Calculated as (total votes) / (total available votes).
    pub quorum: Decimal,
    /// Threshold ratio for a vote option to be the winning one.
    /// Calculated as (votes for certain option) / (total available votes - abstaining votes).
    pub threshold: Decimal,

    #[schemars(with = "Vec<(u8, Uint128)>")]
    #[serde_as(as = "Vec<(_, _)>")]
    /// Total vote-count (value) for each outcome (key).
    pub results: BTreeMap<u8, u128>,
}

/// <poll_id, poll>
pub type Polls<'a> = IndexedMap<'a, PollId, Poll, PollIndices<'a>>;

pub struct PollIndices<'a> {
    /// pk(poll_id)

    /// ik(poll_status)
    pub poll_status: MultiIndex<'a, Vec<u8>, Poll, PollId>,
}

impl<'a> IndexList<Poll> for PollIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Poll>> + '_> {
        let v: Vec<&dyn Index<Poll>> = vec![&self.poll_status];
        Box::new(v.into_iter())
    }
}

pub fn polls<'a>() -> IndexedMap<'a, PollId, Poll, PollIndices<'a>> {
    let indices = PollIndices {
        poll_status: MultiIndex::new(
            |_, d| {
                to_binary(&d.status.clone().to_filter())
                    .expect("error serializing poll status")
                    .0
            },
            "POLLS",
            "POLLS__POLL_STATUS",
        ),
    };
    IndexedMap::new("POLLS", indices)
}

pub trait PollStorage {
    /// Saves a poll using pk(poll_id)
    ///
    /// # Example
    ///
    /// ```
    /// # use cosmwasm_std::{Addr, Decimal, testing::mock_dependencies, Timestamp};
    /// # use poll_engine::error::PollResult;
    /// # use poll_engine::api::VotingScheme;
    /// # use poll_engine::state::{GOV_STATE, GovState, Poll, polls, PollStorage};
    /// # use cosmwasm_std::Uint128;
    /// # fn main() -> PollResult<()> {
    /// # let mut deps = mock_dependencies();
    /// # let state = GovState::default();
    /// # GOV_STATE.save(&mut deps.storage, &state).unwrap();
    /// let poll_id = 123;
    /// let quorum = Decimal::from_ratio(3u8, 10u8);
    /// let threshold = Decimal::percent(50);
    /// let initial_poll = Poll::new(
    ///     &mut deps.as_mut(),
    ///     Addr::unchecked("proposer"),
    ///     10000,
    ///     "some label",
    ///     "some description",
    ///     VotingScheme::CoinVoting,
    ///     Timestamp::from_seconds(5),
    ///     Timestamp::from_seconds(10),
    ///     quorum,
    ///     threshold,
    /// )?;
    /// polls().save(&mut deps.storage, poll_id, &initial_poll)?;
    /// let loaded_poll = polls().load_poll(&deps.storage, poll_id)?;
    ///
    /// assert_eq!(initial_poll, loaded_poll);
    /// # Ok(())
    /// # }
    /// ```
    fn save_poll(&self, store: &mut dyn Storage, poll: Poll) -> PollResult<()>;

    /// Loads a poll using pk(poll_id)
    ///
    /// # Example
    ///
    /// ```
    /// # use cosmwasm_std::{Addr, Decimal, testing::mock_dependencies, Timestamp};
    /// # use poll_engine::error::PollResult;
    /// # fn main() -> PollResult<()> {
    /// # use poll_engine::api::VotingScheme;
    /// # use poll_engine::state::{GOV_STATE, GovState, Poll, polls, PollStorage};
    /// let mut deps = mock_dependencies();
    /// # let state = GovState::default();
    /// # GOV_STATE.save(&mut deps.storage, &state).unwrap();
    /// let poll_id = 123;
    /// let quorum = Decimal::from_ratio(3u8, 10u8);
    /// let threshold = Decimal::percent(50);
    /// let initial_poll = Poll::new(
    ///     &mut deps.as_mut(),
    ///     Addr::unchecked("proposer"),
    ///     10000,
    ///     "some label",
    ///     "some description",
    ///     VotingScheme::CoinVoting,
    ///     Timestamp::from_seconds(5),
    ///     Timestamp::from_seconds(10),
    ///     quorum,
    ///     threshold,
    /// )?;
    /// polls().save(&mut deps.storage, poll_id, &initial_poll)?;
    /// let loaded_poll = polls().load_poll(&mut deps.storage, poll_id)?;
    ///
    /// assert_eq!(initial_poll, loaded_poll);
    /// # Ok(())
    /// # }
    /// ```
    fn load_poll(&self, store: &dyn Storage, key: PollId) -> PollResult<Poll>;
}

impl PollStorage for Polls<'_> {
    fn save_poll(&self, store: &mut dyn Storage, poll: Poll) -> PollResult<()> {
        self.save(store, poll.id, &poll).map_err(PollError::Std)
    }

    fn load_poll(&self, store: &dyn Storage, poll_id: PollId) -> PollResult<Poll> {
        self.may_load(store, poll_id)?
            .ok_or_else(|| PollError::PollNotFound {
                poll_id: poll_id.into(),
            })
    }
}

/// <(voter, poll_id, outcome), vote>
pub type Votes<'a> = IndexedMap<'a, (Addr, PollId), Vote, VoteIndices<'a>>;

pub struct VoteIndices<'a> {
    /// pk(voter, poll_id)

    /// ik(poll_id)
    pub poll: MultiIndex<'a, PollId, Vote, (Addr, PollId)>,
}

impl<'a> IndexList<Vote> for VoteIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Vote>> + '_> {
        let v: Vec<&dyn Index<Vote>> = vec![&self.poll];
        Box::new(v.into_iter())
    }
}

pub fn votes<'a>() -> IndexedMap<'a, (Addr, PollId), Vote, VoteIndices<'a>> {
    let indices = VoteIndices {
        poll: MultiIndex::new(|_, d| d.poll_id, "VOTES", "VOTES__POLL"),
    };
    IndexedMap::new("VOTES", indices)
}

/// Similar to an Option, but can also hold both values in the case of a draw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MostVoted<T> {
    None,
    Some(T),
    Draw(T, T),
}

impl<A, B> MostVoted<(A, B)> {
    pub fn destructure(self) -> (Option<A>, Option<B>) {
        match self {
            MostVoted::None => (None, None),
            MostVoted::Some((outcome, count)) => (Some(outcome), Some(count)),
            MostVoted::Draw(_, _) => (None, None),
        }
    }
}

pub trait VoteStorage {
    /// Saves a voter's vote using pk(voter, poll_id, outcome).
    ///
    /// # Example
    ///
    /// ```
    /// # use cosmwasm_std::{Addr, testing::mock_dependencies};
    /// # use poll_engine::error::PollResult;
    /// # use poll_engine::api::Vote;
    /// # use poll_engine::state::votes;
    /// # use poll_engine::state::VoteStorage;
    /// # use poll_engine::api::VoteOutcome::No;
    /// # fn main() -> PollResult<()> {
    /// let mut deps = mock_dependencies();
    /// let vote = Vote::new(123, Addr::unchecked("voter"), No, 9);
    /// let res = votes().save_vote(&mut deps.storage, vote);
    ///
    /// assert!(res.is_ok());
    /// # Ok(())
    /// # }
    /// ```
    fn save_vote(&self, store: &mut dyn Storage, vote: Vote) -> PollResult<()>;

    /// Returns a voter's votes on a poll using ik(voter, poll_id).
    ///
    /// # Example
    ///
    /// ```
    /// # use cosmwasm_std::{Addr, testing::mock_dependencies};
    /// # use common::cw::*;
    /// # use poll_engine::error::PollResult;
    /// # use poll_engine::api::Vote;
    /// # use poll_engine::state::{votes, VoteStorage};
    /// # use poll_engine::api::VoteOutcome::{Yes, No};
    /// # fn main() -> PollResult<()> {
    /// let mut deps = mock_dependencies();
    /// votes().save_vote(&mut deps.storage, Vote::new(123, Addr::unchecked("voter"), No, 9))?;
    /// votes().save_vote(&mut deps.storage, Vote::new(123, Addr::unchecked("voter"), Yes, 3))?;
    /// let voter_vote = votes().poll_voter(
    ///     &deps.storage, 123,
    ///     Addr::unchecked("voter")
    /// )?;
    ///
    /// assert_eq!(
    ///     Some(Vote::new(123, Addr::unchecked("voter"), Yes, 3)),
    ///     voter_vote
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn poll_voter(
        &self,
        store: &dyn Storage,
        poll_id: u64,
        voter: Addr,
    ) -> PollResult<Option<Vote>>;

    fn poll_voters(
        &self,
        store: &dyn Storage,
        range_args: RangeArgs<(Addr, PollId)>,
    ) -> PollResult<Vec<(u8, u128)>>;

    /// Returns a voter's max vote on any poll with the provided poll status using ik(poll_status).
    ///
    /// # Example
    ///
    /// ```
    /// # use cosmwasm_std::{Addr, testing::mock_dependencies};
    /// # use common::cw::*;
    /// # use poll_engine::error::PollResult;
    /// # use poll_engine::helpers::mock_poll_with_id;
    /// # use poll_engine::api::VoteOutcome::{Abstain, Yes};
    /// # fn main() -> PollResult<()> {
    /// # use common::cw::RangeArgs;
    /// # use poll_engine::api::{PollStatusFilter, Vote};
    /// # use poll_engine::state::{polls, PollStorage, votes, VoteStorage};
    /// # let mut deps = mock_dependencies();
    /// # let voter = Addr::unchecked("voter");
    /// # polls().save_poll(&mut deps.storage, mock_poll_with_id(123))?;
    /// # polls().save_poll(&mut deps.storage, mock_poll_with_id(456))?;
    /// # votes().save_vote(&mut deps.storage, Vote::new(123, voter.clone(), Yes, 10));
    /// # votes().save_vote(&mut deps.storage, Vote::new(456, voter.clone(), Abstain, 20));
    /// let max = votes().max_vote(
    ///     &deps.storage, Addr::unchecked("voter"),
    ///     PollStatusFilter::InProgress,
    ///     RangeArgs::default(),
    ///     RangeArgs::default(),
    /// )?;
    ///
    /// assert_eq!(Some(Vote::new(456, voter.clone(), Abstain, 20)), max);
    /// # Ok(())
    /// # }
    /// ```
    fn max_vote(
        &self,
        store: &dyn Storage,
        voter: Addr,
        poll_status: PollStatusFilter,
        poll_range_args: RangeArgs<PollId>,
        voter_range_args: RangeArgs<PollId>,
    ) -> PollResult<Option<Vote>>;
}

impl VoteStorage for Votes<'_> {
    fn save_vote(&self, store: &mut dyn Storage, vote: Vote) -> PollResult<()> {
        let Vote { voter, poll_id, .. } = vote.clone();
        let key = (voter, poll_id);
        self.save(store, key, &vote).map_err(PollError::Std)
    }

    fn poll_voter(
        &self,
        store: &'_ dyn Storage,
        poll_id: u64,
        voter: Addr,
    ) -> PollResult<Option<Vote>> {
        let key = (voter, poll_id);
        Ok(votes().may_load(store, key)?)
    }

    fn poll_voters(
        &self,
        store: &'_ dyn Storage,
        RangeArgs { min, max, order }: RangeArgs<(Addr, PollId)>,
    ) -> PollResult<Vec<(u8, u128)>> {
        let poll_voter_votes = votes()
            .range(store, min, max, order)
            .map(|res| match res {
                Ok((_, vote)) => Ok((vote.outcome, vote.amount)),
                Err(e) => Err(PollError::Std(e)),
            })
            .try_collect()?;

        Ok(poll_voter_votes)
    }

    fn max_vote(
        &self,
        store: &dyn Storage,
        voter: Addr,
        poll_status: PollStatusFilter,
        poll_range_args: RangeArgs<PollId>,
        voter_range_args: RangeArgs<PollId>,
    ) -> PollResult<Option<Vote>> {
        let key = to_binary(&poll_status)?.0;
        let poll_ids: Vec<u64> = polls()
            .idx
            .poll_status
            .prefix(key)
            .range(
                store,
                poll_range_args.min,
                poll_range_args.max,
                poll_range_args.order,
            )
            .map(|res| match res {
                Ok((_, poll)) => Ok(poll.id),
                Err(e) => Err(PollError::Std(e)),
            })
            .try_collect()?;

        let key = voter;
        let max_vote = self
            .prefix(key)
            .range(
                store,
                voter_range_args.min,
                voter_range_args.max,
                voter_range_args.order,
            )
            .flatten()
            .filter(|(_, vote)| poll_ids.contains(&vote.poll_id))
            .map(|(_, vote)| vote)
            .max_by(|a, b| a.amount.cmp(&b.amount));

        Ok(max_vote)
    }
}

impl Poll {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        deps: &mut DepsMut,
        proposer: Addr,
        deposit_amount: u128,
        label: impl Into<String>,
        description: impl Into<String>,
        scheme: VotingScheme,
        started_at: Timestamp,
        ends_at: Timestamp,
        quorum: Decimal,
        threshold: Decimal,
    ) -> PollResult<Self> {
        Ok(Poll {
            id: GOV_STATE.increment_poll_id(deps.storage)?,
            proposer,
            deposit_amount,
            label: label.into(),
            description: description.into(),
            scheme,
            status: PollStatus::InProgress { ends_at },
            started_at,
            ends_at,
            quorum,
            threshold,
            results: BTreeMap::new(),
        })
    }

    /// Creates a poll from a CreatePollRequest model.
    ///
    /// # Example
    ///
    /// ```
    /// # use cosmwasm_std::{Decimal, Timestamp, Uint64};
    /// # use cosmwasm_std::{testing::mock_dependencies, Addr, Uint128};
    /// # use poll_engine::state::GOV_STATE;
    /// # use poll_engine::error::PollResult;
    /// # use poll_engine::api::{CreatePollParams, PollStatus, VotingScheme};
    /// # use poll_engine::state::{GovState, Poll};
    /// # fn main() -> PollResult<()> {
    /// # use common::cw::testing::mock_env;
    /// let mut deps = mock_dependencies();
    /// # let mut env = mock_env();
    /// # let state = GovState::default();
    /// # GOV_STATE.save(&mut deps.storage, &state).unwrap();
    /// # let scheme = VotingScheme::CoinVoting;
    /// # let label = "some_label".to_string();
    /// # let description = "some_description".to_string();
    /// # let poll_id = Uint64::new(1);
    /// # let started_at = Timestamp::from_nanos(2);
    /// # env.block.time = started_at;
    /// # let ends_at = Timestamp::from_nanos(3);
    /// # let quorum = Decimal::from_ratio(3u8, 10u8);
    /// # let threshold = Decimal::percent(50);
    /// # let params = CreatePollParams {
    /// #     proposer: "proposer".to_string(),
    /// #     deposit_amount: Uint128::new(1000),
    /// #     label: label.clone(),
    /// #     description: description.clone(),
    /// #     scheme,
    /// #     ends_at: ends_at.clone(),
    /// #     quorum: quorum.clone(),
    /// #     threshold: threshold.clone(),
    /// # };
    /// # let expected = Poll {
    /// #     id: poll_id.into(),
    /// #     proposer: Addr::unchecked("proposer"),
    /// #     label,
    /// #     description,
    /// #     scheme,
    /// #     status: PollStatus::InProgress {
    /// #         ends_at: ends_at.clone()
    /// #     },
    /// #     started_at,
    /// #     ends_at,
    /// #     quorum,
    /// #     threshold,
    /// #     results: Default::default(),
    /// #     deposit_amount: 1000
    /// # };
    ///
    /// # // let params = CreatePollParams { ... }; // in which for example poll_id=1234
    /// # let actual = Poll::from(&mut deps.as_mut(), &env, params).unwrap();
    ///
    /// # assert_eq!(1, actual.id);
    /// # assert_eq!(expected, actual);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from(deps: &mut DepsMut, env: &Env, params: CreatePollParams) -> PollResult<Self> {
        Poll::new(
            deps,
            deps.api.addr_validate(&params.proposer)?,
            params.deposit_amount.u128(),
            params.label,
            params.description,
            params.scheme,
            env.block.time,
            params.ends_at,
            params.quorum,
            params.threshold,
        )
    }

    /// Increases the count for an outcome in the results map.
    ///
    /// # Example
    ///
    /// ```
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use poll_engine::error::PollResult;
    /// # use poll_engine::state::{GOV_STATE, GovState};
    /// # use poll_engine::helpers::mock_poll;
    /// # fn main() -> PollResult<()> {
    /// use poll_engine::api::VoteOutcome::{No, Yes};
    /// let mut deps = mock_dependencies();
    /// # let state = GovState::default();
    /// # GOV_STATE.save(&mut deps.storage, &state).unwrap();
    /// # let mut poll = mock_poll(&mut deps.storage);
    /// // let poll = Poll::new(...);
    /// poll.increase_results(Yes, 5)?;
    /// poll.increase_results(No, 3)?;
    /// poll.increase_results(No, 6)?;
    ///
    /// assert_eq!(9, *poll.results.get(&(No as u8)).unwrap());
    /// assert_eq!(5, *poll.results.get(&(Yes as u8)).unwrap());
    /// # Ok(())
    /// # }
    /// ```
    pub fn increase_results(
        &mut self,
        outcome: VoteOutcome,
        count: u128,
    ) -> PollResult<Option<u128>> {
        match self.results.get_mut(&(outcome as u8)) {
            Some(total_count) => {
                *total_count += count;
                Ok(Some(*total_count))
            }
            None => Ok(self.results.insert(outcome as u8, count)),
        }
    }

    /// Decreases the count for an outcome in the results map.
    ///
    /// # Example
    ///
    /// ```
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use poll_engine::error::PollResult;
    /// # use poll_engine::helpers::mock_poll;
    /// # fn main() -> PollResult<()> {
    /// # use poll_engine::api::VoteOutcome::{No, Yes};
    /// use poll_engine::state::{GOV_STATE, GovState};
    /// let mut  deps = mock_dependencies();
    /// # let state = GovState::default();
    /// # GOV_STATE.save(&mut deps.storage, &state).unwrap();
    /// # let mut poll = mock_poll(&mut deps.storage);
    /// // let poll = Poll::new(...);
    /// poll.increase_results(Yes, 5);
    /// poll.increase_results(No, 9);
    ///
    /// poll.decrease_results(Yes, 3);
    /// poll.decrease_results(No, 3);
    /// poll.decrease_results(No, 1);
    ///
    /// assert_eq!(2, *poll.results.get(&(Yes as u8)).unwrap());
    /// assert_eq!(5, *poll.results.get(&(No as u8)).unwrap());
    /// # Ok(())
    /// # }
    /// ```
    pub fn decrease_results(&mut self, outcome: VoteOutcome, count: u128) -> Option<u128> {
        match self.results.get_mut(&(outcome as u8)) {
            Some(total_count) => {
                *total_count -= count;
                Some(*total_count)
            }
            None => None,
        }
    }

    /// Determines if the voting threshold has been reached.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::ops::Not;
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use poll_engine::error::PollResult;
    /// # use poll_engine::helpers::mock_poll;
    /// # fn main() -> PollResult<()> {
    /// # use poll_engine::api::VoteOutcome::No;
    /// use poll_engine::state::{GOV_STATE, GovState};
    /// # let mut deps = mock_dependencies();
    /// # let state = GovState::default();
    /// # GOV_STATE.save(&mut deps.storage, &state).unwrap();
    /// # let mut poll = mock_poll(&mut deps.storage);
    /// // let poll = Poll::new(...); // with 50 % threshold
    ///
    /// assert!(poll.threshold_reached().not());
    ///
    /// poll.increase_results(No, 9);
    /// assert!(poll.threshold_reached());
    /// # Ok(())
    /// # }
    /// ```
    pub fn threshold_reached(&self) -> bool {
        match self.most_voted() {
            MostVoted::None => false,
            MostVoted::Some((_, count)) => self.ge_threshold(count),
            MostVoted::Draw((_, a), (_, b)) => self.ge_threshold(a) && self.ge_threshold(b),
        }
    }

    /// Checks if the count-to-total-votes ratio is greater than the threshold.
    fn ge_threshold(&self, count: u128) -> bool {
        Decimal::checked_from_ratio(count, self.total_votes() - self.abstain_votes())
            .unwrap_or(Decimal::zero())
            .ge(&self.threshold)
    }

    /// Determines if the voting quorum has been reached.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::ops::Not;
    /// # use cosmwasm_std::Decimal;
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use poll_engine::error::PollResult;
    /// # use poll_engine::helpers::mock_poll;
    /// # fn main() -> PollResult<()> {
    /// # use poll_engine::api::VoteOutcome::No;
    /// use poll_engine::state::{GOV_STATE, GovState};
    /// # let mut deps = mock_dependencies();
    /// # let state = GovState::default();
    /// # GOV_STATE.save(&mut deps.storage, &state).unwrap();
    /// # let mut poll = mock_poll(&mut deps.storage);
    /// // let poll = Poll::new(...); // with 50 % quorum
    /// let quorum = Decimal::percent(50);
    /// let maximum_available_votes = 20;
    ///
    /// assert!(poll.quorum_reached(&quorum, maximum_available_votes).not());
    ///
    /// poll.increase_results(No, 9);
    /// assert!(poll.quorum_reached(&quorum, maximum_available_votes).not());
    ///
    /// poll.increase_results(No, 1);
    /// assert!(poll.quorum_reached(&quorum, maximum_available_votes));
    /// # Ok(())
    /// # }
    /// ```
    pub fn quorum_reached(&self, quorum: &Decimal, maximum_available_votes: u128) -> bool {
        Decimal::checked_from_ratio(self.total_votes(), maximum_available_votes)
            .unwrap_or(Decimal::zero())
            .ge(quorum)
    }

    /// Returns the total vote count of the poll.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::{collections::BTreeMap, ops::Not};
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use poll_engine::error::PollResult;
    /// # use poll_engine::helpers::mock_poll;
    /// # fn main() -> PollResult<()> {
    /// # use poll_engine::state::{GOV_STATE, GovState};
    /// # let mut deps = mock_dependencies();
    /// # let state = GovState::default();
    /// # GOV_STATE.save(&mut deps.storage, &state).unwrap();
    /// # let mut poll = mock_poll(&mut deps.storage);
    /// # poll.results = BTreeMap::from([(1, 10), (2, 3), (0, 1)]);
    /// // let poll = Poll::new(...); // with the voting results [(1, 10), (2, 3), (0, 1)]
    ///
    /// assert_eq!(14, poll.total_votes());
    /// # Ok(())
    /// # }
    /// ```
    pub fn total_votes(&self) -> u128 {
        self.results.iter().fold(0u128, |acc, i| acc + i.1)
    }

    /// Returns the vote count for abstaining options of the poll.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::{collections::BTreeMap, ops::Not};
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use poll_engine::error::PollResult;
    /// # use poll_engine::helpers::mock_poll;
    /// # fn main() -> PollResult<()> {
    /// # use cosmwasm_std::Decimal;
    /// use poll_engine::state::{GOV_STATE, GovState};
    /// # let mut deps = mock_dependencies();
    /// # let state = GovState::default();
    /// # GOV_STATE.save(&mut deps.storage, &state).unwrap();
    /// # let mut poll = mock_poll(&mut deps.storage);
    /// # poll.results = BTreeMap::from([(1, 10), (2, 3), (0, 1)]);
    ///
    /// assert_eq!(3, poll.abstain_votes());
    /// # Ok(())
    /// # }
    /// ```
    pub fn abstain_votes(&self) -> u128 {
        self.results
            .iter()
            .filter(|result| result.0 == &(Abstain as u8))
            .map(|result| result.1)
            .sum()
    }

    /// Returns the most voted outcome/count, if any.
    /// Does not consider abstaining outcomes.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::{collections::BTreeMap, ops::Not};
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use poll_engine::error::PollResult;
    /// # use poll_engine::helpers::mock_poll;
    /// # fn main() -> PollResult<()> {
    /// # use cosmwasm_std::Decimal;
    /// # use poll_engine::state::{GOV_STATE, GovState, MostVoted};
    /// # let mut deps = mock_dependencies();
    /// # let state = GovState::default();
    /// # GOV_STATE.save(&mut deps.storage, &state).unwrap();
    /// # let mut poll = mock_poll(&mut deps.storage);
    /// # poll.results = BTreeMap::from([(1, 10), (2, 11), (0, 1)]);
    /// // let poll = Poll::new(...); // with the voting results [(1, 10), (2, 3), (0, 1)]
    ///
    /// assert_eq!(MostVoted::Some((1, 10)), poll.most_voted());
    /// # Ok(())
    /// # }
    /// ```
    pub fn most_voted(&self) -> MostVoted<(u8, u128)> {
        // even if there are more than two with the same outcome, it's still a rejection
        let top_two = self
            .results
            .iter()
            .filter(|result| result.0 != &(Abstain as u8))
            .sorted_by(|&(_, a), &(_, b)| b.cmp(a))
            .take(2)
            .map(|(outcome, count)| (*outcome, *count))
            .collect::<Vec<(u8, u128)>>();

        match top_two.as_slice() {
            [] => MostVoted::None,
            [most_voted] => MostVoted::Some(*most_voted),
            [first @ (_, a), second @ (_, b)] if a.eq(b) => MostVoted::Draw(*first, *second),
            [a, ..] => MostVoted::Some(*a),
        }
    }

    /// Determine poll final status when ending a poll.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::{collections::BTreeMap, ops::Not};
    /// # use cosmwasm_std::Uint128;
    /// # use common::cw::testing::mock_ctx;
    /// # use poll_engine::error::PollResult;
    /// # use poll_engine::helpers::mock_poll;
    /// # fn main() -> PollResult<()> {
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use poll_engine::api::{PollRejectionReason, PollStatus};
    /// # use poll_engine::state::{GOV_STATE, GovState};
    /// let mut deps = mock_dependencies();
    /// # let mut ctx = mock_ctx(deps.as_mut());
    /// # let state = GovState::default();
    /// # GOV_STATE.save(ctx.deps.storage, &state).unwrap();
    /// # let mut poll = mock_poll(ctx.deps.storage);
    /// # poll.results = BTreeMap::from([(1, 10), (2, 3), (0, 1)]);
    /// // let poll = Poll::new(...);
    /// // with the voting results [(1, 10), (2, 3), (0, 1)] and rejecting_outcome=1
    ///
    /// assert_eq!(
    ///     PollStatus::Rejected {
    ///         outcome: Some(1),
    ///         count: Some(Uint128::new(10)),
    ///         reason: PollRejectionReason::IsRejectingOutcome
    ///     },
    ///     poll.final_status(100u8.into())?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn final_status(&self, maximum_available_votes: Uint128) -> PollResult<PollStatus> {
        let most_voted = self.most_voted();
        let status = match (
            most_voted,
            self.quorum_reached(&self.quorum, maximum_available_votes.u128()),
            self.threshold_reached(),
        ) {
            (MostVoted::Some(most_voted), true, true)
                if most_voted.0 == No as u8 || most_voted.0 == Veto as u8 =>
            {
                PollStatus::Rejected {
                    outcome: Some(most_voted.0),
                    count: Some(most_voted.1.into()),
                    reason: PollRejectionReason::IsRejectingOutcome,
                }
            }
            (MostVoted::Some(most_voted), true, true) => PollStatus::Passed {
                outcome: most_voted.0,
                count: most_voted.1.into(),
            },
            (MostVoted::Draw(a, b), true, _) => PollStatus::Rejected {
                outcome: None,
                count: None,
                reason: PollRejectionReason::OutcomeDraw(a.0, b.0, b.1.into()),
            },
            (most_voted, false, true) => {
                let (outcome, count) = most_voted.destructure();
                PollStatus::Rejected {
                    outcome,
                    count: count.map(Uint128::new),
                    reason: PollRejectionReason::QuorumNotReached,
                }
            }
            (most_voted, true, false) => {
                let (outcome, count) = most_voted.destructure();
                PollStatus::Rejected {
                    outcome,
                    count: count.map(Uint128::new),
                    reason: PollRejectionReason::ThresholdNotReached,
                }
            }
            (most_voted, false, false) => {
                let (outcome, count) = most_voted.destructure();
                PollStatus::Rejected {
                    outcome,
                    count: count.map(Uint128::new),
                    reason: PollRejectionReason::QuorumAndThresholdNotReached,
                }
            }
            (MostVoted::None, true, true) => unreachable!(),
        };

        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{Decimal, Uint128};

    use common::cw::testing::mock_ctx;

    use crate::api::VoteOutcome::{Abstain, No, Yes};
    use crate::api::{PollRejectionReason, PollStatus};
    use crate::helpers::mock_poll;
    use crate::state::{GovState, GOV_STATE};

    #[test]
    fn final_status_passed() {
        let mut deps = mock_dependencies();
        let ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.quorum = Decimal::percent(10);
        poll.threshold = Decimal::percent(50);
        poll.results = BTreeMap::from([(0, 3), (2, 8), (3, 2)]);

        assert_eq!(
            PollStatus::Passed {
                outcome: Yes as u8,
                count: Uint128::new(3),
            },
            poll.final_status(130u8.into()).unwrap()
        );
    }

    #[test]
    fn final_status_passed_default_type() {
        // TODO: rename?
        let mut deps = mock_dependencies();
        let ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.quorum = Decimal::percent(10);
        poll.results = BTreeMap::from([(No as u8, 2), (Abstain as u8, 8), (Yes as u8, 3)]);

        assert_eq!(
            PollStatus::Passed {
                outcome: Yes as u8,
                count: Uint128::new(3),
            },
            poll.final_status(130u8.into()).unwrap()
        );
    }

    #[test]
    fn final_status_rejected_is_rejecting_outcome() {
        let mut deps = mock_dependencies();
        let ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.quorum = Decimal::percent(10);
        poll.results = BTreeMap::from([(1, 10), (2, 13), (3, 2)]);

        assert_eq!(
            PollStatus::Rejected {
                outcome: Some(1),
                count: Some(Uint128::new(10)),
                reason: PollRejectionReason::IsRejectingOutcome
            },
            poll.final_status(250u8.into()).unwrap()
        );
    }

    #[test]
    fn final_status_rejected_is_rejecting_outcome_default_type() {
        // TODO: rename?
        let mut deps = mock_dependencies();
        let ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.quorum = Decimal::percent(10);
        poll.results = BTreeMap::from([(No as u8, 10), (Abstain as u8, 13), (Yes as u8, 2)]);

        assert_eq!(
            PollStatus::Rejected {
                outcome: Some(1),
                count: Some(Uint128::new(10)),
                reason: PollRejectionReason::IsRejectingOutcome
            },
            poll.final_status(250u8.into()).unwrap()
        );
    }

    #[test]
    fn final_status_rejected_threshold_not_reached() {
        let mut deps = mock_dependencies();
        let ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.results = BTreeMap::from([(1, 5), (3, 6), (4, 5)]);

        assert_eq!(
            PollStatus::Rejected {
                outcome: Some(3),
                count: Some(Uint128::new(6)),
                reason: PollRejectionReason::ThresholdNotReached
            },
            poll.final_status(20u8.into()).unwrap()
        );
    }

    #[test]
    fn final_status_rejected_quorum_not_reached() {
        let mut deps = mock_dependencies();
        let ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.quorum = Decimal::from_ratio(3u8, 10u8);
        poll.results = BTreeMap::from([(1, 1), (2, 3)]);

        assert_eq!(
            PollStatus::Rejected {
                outcome: Some(1),
                count: Some(Uint128::new(1)),
                reason: PollRejectionReason::QuorumNotReached
            },
            poll.final_status(15u8.into()).unwrap()
        );
    }

    #[test]
    fn final_status_rejected_abstained_to_quorum_but_threshold_not_reached() {
        let mut deps = mock_dependencies();
        let ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.quorum = Decimal::percent(50);
        poll.threshold = Decimal::percent(76);
        poll.results = BTreeMap::from([(0, 3), (1, 1), (2, 9)]);

        assert_eq!(
            PollStatus::Rejected {
                outcome: Some(0),
                count: Some(Uint128::new(3)),
                reason: PollRejectionReason::ThresholdNotReached
            },
            poll.final_status(21u8.into()).unwrap()
        );
    }

    #[test]
    fn final_status_rejected_abstained_to_quorum_but_threshold_not_reached_default_type() {
        // TODO: rename?
        let mut deps = mock_dependencies();
        let ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.quorum = Decimal::percent(10);
        poll.threshold = Decimal::percent(76);
        poll.results = BTreeMap::from([(Yes as u8, 3), (No as u8, 1), (Abstain as u8, 9)]);

        assert_eq!(
            PollStatus::Rejected {
                outcome: Some(0),
                count: Some(Uint128::new(3)),
                reason: PollRejectionReason::ThresholdNotReached
            },
            poll.final_status(21u8.into()).unwrap()
        );
    }

    #[test]
    fn final_status_rejected_quorum_and_threshold_not_reached() {
        let mut deps = mock_dependencies();
        let ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.quorum = Decimal::from_ratio(3u8, 10u8);
        poll.results = BTreeMap::from([]);

        assert_eq!(
            PollStatus::Rejected {
                outcome: None,
                count: None,
                reason: PollRejectionReason::QuorumAndThresholdNotReached
            },
            poll.final_status(1u8.into()).unwrap()
        );
    }

    #[test]
    fn final_status_rejected_outcome_draw() {
        let mut deps = mock_dependencies();
        let ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.threshold = Decimal::percent(1);
        poll.results = BTreeMap::from([(1, 5), (2, 6), (0, 5)]);

        assert_eq!(
            PollStatus::Rejected {
                outcome: None,
                count: None,
                reason: PollRejectionReason::OutcomeDraw(0, 1, Uint128::new(5))
            },
            poll.final_status(20u8.into()).unwrap()
        );
    }
}
