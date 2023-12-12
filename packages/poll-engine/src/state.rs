use cosmwasm_schema::cw_serde;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::ops::Not;
use Ordering::{Equal, Greater, Less};

use cosmwasm_std::{to_json_binary, Addr, Decimal, DepsMut, Env, Storage, Timestamp, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
use itertools::Itertools;

use common::cw::RangeArgs;
use PollRejectionReason::{IsVetoOutcome, OutcomeDraw, QuorumNotReached, ThresholdNotReached};

use poll_engine_api::api::PollRejectionReason::IsRejectingOutcome;
use poll_engine_api::api::VoteOutcome::{Abstain, No, Veto, Yes};
use poll_engine_api::api::{
    CreatePollParams, Poll, PollId, PollRejectionReason, PollStatus, PollStatusFilter, Vote,
    VoteOutcome, VotingScheme,
};
use poll_engine_api::error::*;

pub const GOV_STATE: Item<GovState> = Item::new("gov_state");

#[derive(Default)]
#[cw_serde]
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
                to_json_binary(&d.status.clone().to_filter())
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
    /// # use poll_engine_api::error::PollResult;
    /// # use poll_engine_api::api::VotingScheme;
    /// # use poll_engine::state::{GOV_STATE, GovState, polls, PollStorage};
    /// # use cosmwasm_std::Uint128;
    /// # fn main() -> PollResult<()> {
    /// # use poll_engine::state::{new_poll, PollHelpers};
    /// # let mut deps = mock_dependencies();
    /// # let state = GovState::default();
    /// # GOV_STATE.save(&mut deps.storage, &state).unwrap();
    /// let poll_id = 123;
    /// let quorum = Decimal::from_ratio(3u8, 10u8);
    /// let threshold = Decimal::percent(50);
    /// let veto_threshold = Some(Decimal::percent(33));
    /// let initial_poll = new_poll(
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
    ///     veto_threshold,
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
    /// # use poll_engine_api::error::PollResult;
    /// # fn main() -> PollResult<()> {
    /// # use poll_engine_api::api::VotingScheme;
    /// # use poll_engine::state::{GOV_STATE, GovState, new_poll, PollHelpers, polls, PollStorage};
    /// let mut deps = mock_dependencies();
    /// # let state = GovState::default();
    /// # GOV_STATE.save(&mut deps.storage, &state).unwrap();
    /// let poll_id = 123;
    /// let quorum = Decimal::from_ratio(3u8, 10u8);
    /// let threshold = Decimal::percent(50);
    /// let veto_threshold = Some(Decimal::percent(33));
    /// let initial_poll = new_poll(
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
    ///     veto_threshold,
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
    /// # use poll_engine_api::error::PollResult;
    /// # use poll_engine_api::api::Vote;
    /// # use poll_engine::state::votes;
    /// # use poll_engine::state::VoteStorage;
    /// # use poll_engine_api::api::VoteOutcome::No;
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
    /// # use poll_engine_api::error::PollResult;
    /// # use poll_engine_api::api::Vote;
    /// # use poll_engine::state::{votes, VoteStorage};
    /// # use poll_engine_api::api::VoteOutcome::{Yes, No};
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
    /// # use poll_engine_api::error::PollResult;
    /// # use poll_engine::helpers::mock_poll_with_id;
    /// # use poll_engine_api::api::VoteOutcome::{Abstain, Yes};
    /// # fn main() -> PollResult<()> {
    /// # use common::cw::RangeArgs;
    /// # use poll_engine_api::api::{PollStatusFilter, Vote};
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
        let key = to_json_binary(&poll_status)?.0;
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
#[allow(clippy::too_many_arguments)]
pub fn new_poll(
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
    veto_threshold: Option<Decimal>,
) -> PollResult<Poll> {
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
        veto_threshold,
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
/// # use poll_engine_api::error::PollResult;
/// # use poll_engine_api::api::{CreatePollParams, PollStatus, VotingScheme};
/// # use poll_engine::state::{GovState};
/// # fn main() -> PollResult<()> {
/// # use common::cw::testing::mock_env;
/// use poll_engine::state::poll_from;
/// use poll_engine_api::api::Poll;
/// # use crate::poll_engine::state::PollHelpers;
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
/// # let veto_threshold = Some(Decimal::percent(33));
/// # let params = CreatePollParams {
/// #     proposer: "proposer".to_string(),
/// #     deposit_amount: Uint128::new(1000),
/// #     label: label.clone(),
/// #     description: description.clone(),
/// #     scheme,
/// #     ends_at: ends_at.clone(),
/// #     quorum: quorum.clone(),
/// #     threshold: threshold.clone(),
/// #     veto_threshold: veto_threshold.clone(),
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
/// #     veto_threshold,
/// #     results: Default::default(),
/// #     deposit_amount: 1000
/// # };
///
/// # // let params = CreatePollParams { ... }; // in which for example poll_id=1234
/// # let actual = poll_from(&mut deps.as_mut(), &env, params).unwrap();
///
/// # assert_eq!(1, actual.id);
/// # assert_eq!(expected, actual);
/// # Ok(())
/// # }
/// ```
pub fn poll_from(deps: &mut DepsMut, env: &Env, params: CreatePollParams) -> PollResult<Poll> {
    new_poll(
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
        params.veto_threshold,
    )
}

pub trait PollHelpers {
    fn increase_results(&mut self, outcome: VoteOutcome, count: u128) -> PollResult<Option<u128>>;

    fn decrease_results(&mut self, outcome: VoteOutcome, count: u128) -> Option<u128>;

    fn threshold_reached(&self, outcome: VoteOutcome) -> bool;

    fn ge_threshold(&self, outcome: VoteOutcome, count: u128) -> bool;

    fn quorum_reached(&self, quorum: &Decimal, maximum_available_votes: u128) -> bool;

    fn total_votes(&self) -> u128;

    fn votes_for(&self, outcome: VoteOutcome) -> u128;

    fn most_voted_over_threshold(&self) -> MostVoted<(u8, u128)>;

    fn final_status(&self, maximum_available_votes: Uint128) -> PollResult<PollStatus>;
}

impl PollHelpers for Poll {
    /// Increases the count for an outcome in the results map.
    ///
    /// # Example
    ///
    /// ```
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use poll_engine_api::error::PollResult;
    /// # use poll_engine::state::{GOV_STATE, GovState};
    /// # use poll_engine::helpers::mock_poll;
    /// # fn main() -> PollResult<()> {
    /// use poll_engine::state::PollHelpers;
    /// use poll_engine_api::api::VoteOutcome::{No, Yes};
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
    fn increase_results(&mut self, outcome: VoteOutcome, count: u128) -> PollResult<Option<u128>> {
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
    /// # use poll_engine_api::error::PollResult;
    /// # use poll_engine::helpers::mock_poll;
    /// # fn main() -> PollResult<()> {
    /// # use poll_engine_api::api::VoteOutcome::{No, Yes};
    /// use poll_engine::state::{GOV_STATE, GovState, PollHelpers};
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
    fn decrease_results(&mut self, outcome: VoteOutcome, count: u128) -> Option<u128> {
        match self.results.get_mut(&(outcome as u8)) {
            Some(total_count) => {
                *total_count -= count;
                Some(*total_count)
            }
            None => None,
        }
    }

    /// Determines if the voting threshold has been reached for a specific vote outcome.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::ops::Not;
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use poll_engine_api::error::PollResult;
    /// # use poll_engine::helpers::mock_poll;
    /// # fn main() -> PollResult<()> {
    /// # use poll_engine_api::api::VoteOutcome::{No, Yes};
    /// use poll_engine::state::{GOV_STATE, GovState, PollHelpers};
    /// # let mut deps = mock_dependencies();
    /// # let state = GovState::default();
    /// # GOV_STATE.save(&mut deps.storage, &state).unwrap();
    /// # let mut poll = mock_poll(&mut deps.storage);
    /// // let poll = Poll::new(...); // with 50 % threshold
    ///
    /// assert!(poll.threshold_reached(No).not());
    /// assert!(poll.threshold_reached(Yes).not());
    ///
    /// poll.increase_results(No, 9);
    /// assert!(poll.threshold_reached(No));
    /// assert!(poll.threshold_reached(Yes).not());
    /// # Ok(())
    /// # }
    /// ```
    fn threshold_reached(&self, outcome: VoteOutcome) -> bool {
        self.ge_threshold(outcome, self.votes_for(outcome))
    }

    /// Checks if the count-to-total-votes ratio is greater than the threshold.
    fn ge_threshold(&self, outcome: VoteOutcome, count: u128) -> bool {
        let threshold = if outcome == Veto {
            self.veto_threshold.unwrap_or(self.threshold)
        } else {
            self.threshold
        };
        Decimal::checked_from_ratio(count, self.total_votes() - self.votes_for(Abstain))
            .unwrap_or(Decimal::zero())
            .ge(&threshold)
    }

    /// Determines if the voting quorum has been reached.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::ops::Not;
    /// # use cosmwasm_std::Decimal;
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use poll_engine_api::error::PollResult;
    /// # use poll_engine::helpers::mock_poll;
    /// # fn main() -> PollResult<()> {
    /// # use poll_engine_api::api::VoteOutcome::No;
    /// use poll_engine::state::{GOV_STATE, GovState, PollHelpers};
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
    fn quorum_reached(&self, quorum: &Decimal, maximum_available_votes: u128) -> bool {
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
    /// # use poll_engine_api::error::PollResult;
    /// # use poll_engine::helpers::mock_poll;
    /// # fn main() -> PollResult<()> {
    /// # use poll_engine::state::{GOV_STATE, GovState, PollHelpers};
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
    fn total_votes(&self) -> u128 {
        self.results.iter().fold(0u128, |acc, i| acc + i.1)
    }

    /// Returns the vote count for specific vote outcome.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::{collections::BTreeMap, ops::Not};
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use poll_engine_api::error::PollResult;
    /// # use poll_engine::helpers::mock_poll;
    /// # use poll_engine_api::api::VoteOutcome::{Abstain, No, Yes, Veto};
    /// # fn main() -> PollResult<()> {
    /// # use cosmwasm_std::Decimal;
    /// use poll_engine::state::{GOV_STATE, GovState, PollHelpers};
    /// # let mut deps = mock_dependencies();
    /// # let state = GovState::default();
    /// # GOV_STATE.save(&mut deps.storage, &state).unwrap();
    /// # let mut poll = mock_poll(&mut deps.storage);
    /// # poll.results = BTreeMap::from([(No as u8, 10), (Abstain as u8, 3), (Yes as u8, 1)]);
    ///
    /// assert_eq!(1, poll.votes_for(Yes));
    /// assert_eq!(10, poll.votes_for(No));
    /// assert_eq!(3, poll.votes_for(Abstain));
    /// assert_eq!(0, poll.votes_for(Veto));
    /// # Ok(())
    /// # }
    /// ```
    fn votes_for(&self, outcome: VoteOutcome) -> u128 {
        *self.results.get(&(outcome as u8)).unwrap_or(&0u128)
    }

    /// Returns the most voted outcome/count, if any.
    /// Does not consider abstaining outcomes.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::{collections::BTreeMap, ops::Not};
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use poll_engine_api::error::PollResult;
    /// # use poll_engine::helpers::mock_poll;
    /// # fn main() -> PollResult<()> {
    /// # use cosmwasm_std::Decimal;
    /// # use poll_engine::state::{GOV_STATE, GovState, MostVoted, PollHelpers};
    /// # let mut deps = mock_dependencies();
    /// # let state = GovState::default();
    /// # GOV_STATE.save(&mut deps.storage, &state).unwrap();
    /// # let mut poll = mock_poll(&mut deps.storage);
    /// # poll.results = BTreeMap::from([(1, 10), (2, 11), (0, 1)]);
    /// // let poll = Poll::new(...); // with the voting results [(1, 10), (2, 3), (0, 1)]
    ///
    /// assert_eq!(MostVoted::Some((1, 10)), poll.most_voted_over_threshold());
    /// # Ok(())
    /// # }
    /// ```
    fn most_voted_over_threshold(&self) -> MostVoted<(u8, u128)> {
        if self.threshold_reached(Veto) {
            // if veto threshold reached, no need to check anything else
            return MostVoted::Some((Veto as u8, self.votes_for(Veto)));
        };

        match (self.threshold_reached(Yes), self.threshold_reached(No)) {
            (true, true) => {
                let yes_votes = self.votes_for(Yes);
                let no_votes = self.votes_for(No);

                match yes_votes.cmp(&no_votes) {
                    Less => MostVoted::Some((No as u8, no_votes)),
                    Equal => MostVoted::Draw((Yes as u8, yes_votes), (No as u8, no_votes)),
                    Greater => MostVoted::Some((Yes as u8, yes_votes)),
                }
            }
            (true, false) => MostVoted::Some((Yes as u8, self.votes_for(Yes))),
            (false, true) => MostVoted::Some((No as u8, self.votes_for(No))),
            (false, false) => MostVoted::None,
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
    /// # use poll_engine_api::error::PollResult;
    /// # use poll_engine::helpers::mock_poll;
    /// # fn main() -> PollResult<()> {
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use poll_engine_api::api::{PollRejectionReason, PollStatus};
    /// # use poll_engine::state::{GOV_STATE, GovState, PollHelpers};
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
    ///         reason: PollRejectionReason::IsRejectingOutcome
    ///     },
    ///     poll.final_status(100u8.into())?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn final_status(&self, maximum_available_votes: Uint128) -> PollResult<PollStatus> {
        let status = if self
            .quorum_reached(&self.quorum, maximum_available_votes.u128())
            .not()
        {
            PollStatus::Rejected {
                reason: QuorumNotReached,
            }
        } else {
            let most_voted = self.most_voted_over_threshold();
            match most_voted {
                MostVoted::None => PollStatus::Rejected {
                    reason: ThresholdNotReached,
                },
                MostVoted::Some((outcome, count)) => {
                    if outcome == Yes as u8 {
                        PollStatus::Passed {
                            outcome,
                            count: count.into(),
                        }
                    } else if outcome == Veto as u8 {
                        PollStatus::Rejected {
                            reason: IsVetoOutcome,
                        }
                    } else {
                        PollStatus::Rejected {
                            reason: IsRejectingOutcome,
                        }
                    }
                }
                MostVoted::Draw(a, b) => PollStatus::Rejected {
                    reason: OutcomeDraw(a.0, b.0, b.1.into()),
                },
            }
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
    use poll_engine_api::api::PollRejectionReason::{
        IsVetoOutcome, OutcomeDraw, QuorumNotReached, ThresholdNotReached,
    };
    use poll_engine_api::api::VoteOutcome::{Abstain, No, Veto, Yes};

    use crate::helpers::mock_poll;
    use crate::state::{GovState, PollHelpers, GOV_STATE};
    use poll_engine_api::api::{PollRejectionReason, PollStatus};

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
    fn final_status_rejected_is_rejecting_outcome() {
        let mut deps = mock_dependencies();
        let ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.quorum = Decimal::percent(10);
        poll.results = BTreeMap::from([(No as u8, 10), (Abstain as u8, 13), (Veto as u8, 2)]);

        assert_eq!(
            PollStatus::Rejected {
                reason: PollRejectionReason::IsRejectingOutcome
            },
            poll.final_status(250u8.into()).unwrap()
        );
    }

    #[test]
    fn final_status_rejected_is_veto_outcome() {
        let mut deps = mock_dependencies();
        let ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.quorum = Decimal::percent(10);
        poll.results = BTreeMap::from([(No as u8, 2), (Abstain as u8, 13), (Veto as u8, 10)]);

        assert_eq!(
            PollStatus::Rejected {
                reason: IsVetoOutcome,
            },
            poll.final_status(250u8.into()).unwrap()
        );
    }

    #[test]
    fn final_status_rejected_is_veto_outcome_respects_veto_threshold() {
        let mut deps = mock_dependencies();
        let ctx = mock_ctx(deps.as_mut());
        let state = GovState::default();
        GOV_STATE.save(ctx.deps.storage, &state).unwrap();

        let mut poll = mock_poll(ctx.deps.storage);
        poll.quorum = Decimal::percent(10);
        poll.threshold = Decimal::percent(50);
        poll.veto_threshold = Some(Decimal::percent(33));
        poll.results = BTreeMap::from([
            (Yes as u8, 4),
            (No as u8, 2),
            (Abstain as u8, 13),
            (Veto as u8, 3),
        ]);

        assert_eq!(
            PollStatus::Rejected {
                reason: IsVetoOutcome,
            },
            poll.final_status(220u8.into()).unwrap()
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
                reason: ThresholdNotReached
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
                reason: QuorumNotReached
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
                reason: ThresholdNotReached
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
                reason: QuorumNotReached
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
                reason: OutcomeDraw(0, 1, Uint128::new(5))
            },
            poll.final_status(20u8.into()).unwrap()
        );
    }
}
