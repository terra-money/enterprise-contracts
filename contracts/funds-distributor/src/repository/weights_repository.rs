use crate::distributing::query_enterprise_components;
use crate::eligibility::MINIMUM_ELIGIBLE_WEIGHT;
use crate::participation::get_proposal_ids_tracked;
use crate::repository::era_repository::{get_current_era, get_user_first_era_with_weight};
use crate::state::EraId;
use crate::user_weights::{EFFECTIVE_USER_WEIGHTS, USER_WEIGHTS};
use cosmwasm_std::{Addr, Deps, DepsMut, StdResult, Uint128};
use cosmwasm_std::Order::{Ascending, Descending};
use cw_storage_plus::{Map, PrefixBound};
use enterprise_governance_api::msg::QueryMsg::{TotalVotes, VoterTotalVotes};
use funds_distributor_api::api::DistributionType;
use funds_distributor_api::api::DistributionType::Membership;
use funds_distributor_api::error::DistributorResult;
use poll_engine_api::api::{
    TotalVotesParams, TotalVotesResponse, VoterTotalVotesParams, VoterTotalVotesResponse,
};
use DistributionType::Participation;

// TODO: this is actually for membership only
/// Total weight of all users eligible for rewards for the given era.
const ERA_EFFECTIVE_TOTAL_WEIGHT: Map<EraId, Uint128> = Map::new("era_total_weight");

const PARTICIPATION_TOTAL_WEIGHT: Map<EraId, Uint128> = Map::new("participation_total_weight");

pub trait WeightsRepository {
    fn get_total_weight(&self, era_id: EraId) -> DistributorResult<Uint128>;

    // TODO: it gets confusing whether this queries live data or uses some local copy, split into interactor and repository
    fn get_user_weight(&self, user: Addr, era_id: EraId) -> DistributorResult<Option<Uint128>>;
}

pub trait WeightsRepositoryMut<'a>: WeightsRepository {
    fn set_total_weight(&mut self, total_weight: Uint128, era_id: EraId) -> DistributorResult<()>;

    /// Sets a user's weight to a new value.
    /// Returns the user's new effective weight that will be applied to them.
    fn set_user_weight(&mut self, user: Addr, weight: Uint128) -> DistributorResult<Uint128>;
}

////////////////////////////////
////////// MEMBERSHIP //////////
////////////////////////////////

pub struct MembershipWeightsRepository<'a> {
    deps: Deps<'a>,
}

impl WeightsRepository for MembershipWeightsRepository<'_> {
    fn get_total_weight(&self, era_id: EraId) -> DistributorResult<Uint128> {
        let total_weight = ERA_EFFECTIVE_TOTAL_WEIGHT
            .may_load(self.deps.storage, era_id)?
            .unwrap_or_default();
        Ok(total_weight)
    }

    fn get_user_weight(&self, user: Addr, era_id: EraId) -> DistributorResult<Option<Uint128>> {
        let weight = USER_WEIGHTS.may_load(self.deps.storage, (era_id, user.clone()))?;

        match weight {
            Some(weight) => {
                let minimum_eligible_weight =
                    MINIMUM_ELIGIBLE_WEIGHT.load(self.deps.storage, era_id)?;

                let effective_weight = calculate_effective_weight(weight, minimum_eligible_weight);

                Ok(Some(effective_weight))
            }
            None => {
                let first_era_with_weight = get_user_first_era_with_weight(self.deps.clone(), user.clone(), Membership)?;

                USER_WEIGHTS.range(self.deps.storage, None, None, Ascending)
                    .collect::<StdResult<Vec<((EraId, Addr), Uint128)>>>()?
                    .iter().for_each(|&(((era, _), weight))| println!("user: {}, era: {}, weight: {}", user.to_string(), era, weight));

                if let Some(first_era_with_weight) = first_era_with_weight {
                    let weight = USER_WEIGHTS.prefix_range(
                        self.deps.storage, Some(PrefixBound::exclusive(era_id)), Some(PrefixBound::inclusive(first_era_with_weight)), Descending,
                    )
                        .take(1)
                        .collect::<StdResult<Vec<((EraId, Addr), Uint128)>>>()?
                        .first()
                        .map(|&(_, weight)| weight);
                    Ok(weight)
                } else {
                    Ok(None)
                }
            }
        }
    }
}

pub struct MembershipWeightsRepositoryMut<'a> {
    deps: DepsMut<'a>,
}

impl MembershipWeightsRepositoryMut<'_> {
    pub fn as_ref(&self) -> MembershipWeightsRepository {
        MembershipWeightsRepository {
            deps: self.deps.as_ref(),
        }
    }
}

impl WeightsRepository for MembershipWeightsRepositoryMut<'_> {
    fn get_total_weight(&self, era_id: EraId) -> DistributorResult<Uint128> {
        self.as_ref().get_total_weight(era_id)
    }

    fn get_user_weight(&self, user: Addr, era_id: EraId) -> DistributorResult<Option<Uint128>> {
        self.as_ref().get_user_weight(user, era_id)
    }
}

impl<'a> WeightsRepositoryMut<'a> for MembershipWeightsRepositoryMut<'a> {
    fn set_total_weight(&mut self, total_weight: Uint128, era_id: EraId) -> DistributorResult<()> {
        ERA_EFFECTIVE_TOTAL_WEIGHT.save(self.deps.storage, era_id, &total_weight)?;
        Ok(())
    }

    fn set_user_weight(&mut self, user: Addr, weight: Uint128) -> DistributorResult<Uint128> {
        let current_era = get_current_era(self.deps.as_ref(), Membership)?;
        let minimum_eligible_weight =
            MINIMUM_ELIGIBLE_WEIGHT.load(self.deps.storage, current_era)?;

        USER_WEIGHTS.save(self.deps.storage, (current_era, user.clone()), &weight)?;

        let effective_user_weight = calculate_effective_weight(weight, minimum_eligible_weight);
        EFFECTIVE_USER_WEIGHTS.save(self.deps.storage, user, &effective_user_weight)?;

        Ok(effective_user_weight)
    }
}

// TODO: this is a weird file for this function to live in, move somewhere better
pub fn calculate_effective_weight(weight: Uint128, minimum_eligible_weight: Uint128) -> Uint128 {
    if weight >= minimum_eligible_weight {
        weight
    } else {
        Uint128::zero()
    }
}

///////////////////////////////////
////////// PARTICIPATION //////////
///////////////////////////////////

pub struct ParticipationWeightsRepository<'a> {
    deps: Deps<'a>,
}

impl WeightsRepository for ParticipationWeightsRepository<'_> {
    fn get_total_weight(&self, era_id: EraId) -> DistributorResult<Uint128> {
        let components = query_enterprise_components(self.deps)?;

        let tracked_proposal_ids = get_proposal_ids_tracked(self.deps, era_id)?;

        let total_weight: TotalVotesResponse = self.deps.querier.query_wasm_smart(
            components.enterprise_governance_contract.to_string(),
            &TotalVotes(TotalVotesParams {
                poll_ids: tracked_proposal_ids,
            }),
        )?;
        // TODO: revert to this (loading of local weight), but we need to figure out how to ensure it's fresh
        // TODO: the reason it wasn't fresh is because we're setting it prior to user votes being registered, meaning it doesn't account for the vote change
        // let total_weight = PARTICIPATION_TOTAL_WEIGHT
        //     .may_load(self.deps.storage, era_id)?
        //     .unwrap_or_default();
        Ok(total_weight.total_votes)
    }

    // TODO: this should go to interactor, not here
    fn get_user_weight(&self, user: Addr, era_id: EraId) -> DistributorResult<Option<Uint128>> {
        let components = query_enterprise_components(self.deps)?;

        let tracked_proposal_ids = get_proposal_ids_tracked(self.deps, era_id)?;

        let user_weight: VoterTotalVotesResponse = self.deps.querier.query_wasm_smart(
            components.enterprise_governance_contract.to_string(),
            &VoterTotalVotes(VoterTotalVotesParams {
                voter_addr: user.to_string(),
                poll_ids: tracked_proposal_ids,
            }),
        )?;

        Ok(user_weight.total_votes)
    }
}

pub struct ParticipationWeightsRepositoryMut<'a> {
    deps: DepsMut<'a>,
}

impl ParticipationWeightsRepositoryMut<'_> {
    pub fn as_ref(&self) -> ParticipationWeightsRepository {
        ParticipationWeightsRepository {
            deps: self.deps.as_ref(),
        }
    }
}

impl WeightsRepository for ParticipationWeightsRepositoryMut<'_> {
    fn get_total_weight(&self, era_id: EraId) -> DistributorResult<Uint128> {
        self.as_ref().get_total_weight(era_id)
    }

    fn get_user_weight(&self, user: Addr, era_id: EraId) -> DistributorResult<Option<Uint128>> {
        self.as_ref().get_user_weight(user, era_id)
    }
}

impl<'a> WeightsRepositoryMut<'a> for ParticipationWeightsRepositoryMut<'a> {
    fn set_total_weight(&mut self, total_weight: Uint128, era_id: EraId) -> DistributorResult<()> {
        PARTICIPATION_TOTAL_WEIGHT.save(self.deps.storage, era_id, &total_weight)?;
        Ok(())
    }

    fn set_user_weight(&mut self, _user: Addr, _weight: Uint128) -> DistributorResult<Uint128> {
        todo!("probably no-op")
    }
}

pub fn weights_repository<'a>(
    deps: Deps<'a>,
    distribution_type: DistributionType,
) -> Box<dyn WeightsRepository + 'a> {
    match distribution_type {
        Membership => Box::new(MembershipWeightsRepository { deps }),
        Participation => Box::new(ParticipationWeightsRepository { deps }),
    }
}

pub fn weights_repository_mut<'a>(
    deps: DepsMut<'a>,
    distribution_type: DistributionType,
) -> Box<dyn WeightsRepositoryMut + 'a> {
    match distribution_type {
        Membership => Box::new(MembershipWeightsRepositoryMut { deps }),
        Participation => Box::new(ParticipationWeightsRepositoryMut { deps }),
    }
}
