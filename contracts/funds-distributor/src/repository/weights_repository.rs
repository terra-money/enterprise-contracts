use crate::distributing::query_enterprise_components;
use crate::eligibility::MINIMUM_ELIGIBLE_WEIGHT;
use crate::participation::{get_proposal_ids_tracked, PARTICIPATION_TOTAL_WEIGHT};
use crate::state::EFFECTIVE_TOTAL_WEIGHT;
use crate::user_weights::{EFFECTIVE_USER_WEIGHTS, USER_WEIGHTS};
use cosmwasm_std::{Addr, Deps, DepsMut, Uint128};
use enterprise_governance_api::msg::QueryMsg::VoterTotalVotes;
use funds_distributor_api::api::DistributionType;
use funds_distributor_api::error::DistributorResult;
use poll_engine_api::api::{VoterTotalVotesParams, VoterTotalVotesResponse};

pub trait WeightsRepository {
    fn get_total_weight(&self) -> DistributorResult<Uint128>;

    // TODO: it gets confusing whether this queries live data or uses some local copy, split into interactor and repository
    fn get_user_weight(&self, user: Addr) -> DistributorResult<Option<Uint128>>;
}

pub trait WeightsRepositoryMut<'a>: WeightsRepository {
    fn set_total_weight(&mut self, total_weight: Uint128) -> DistributorResult<()>;

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
    fn get_total_weight(&self) -> DistributorResult<Uint128> {
        let total_weight = EFFECTIVE_TOTAL_WEIGHT
            .may_load(self.deps.storage)?
            .unwrap_or_default();
        Ok(total_weight)
    }

    fn get_user_weight(&self, user: Addr) -> DistributorResult<Option<Uint128>> {
        let user_weight = EFFECTIVE_USER_WEIGHTS.may_load(self.deps.storage, user)?;
        Ok(user_weight)
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
    fn get_total_weight(&self) -> DistributorResult<Uint128> {
        self.as_ref().get_total_weight()
    }

    fn get_user_weight(&self, user: Addr) -> DistributorResult<Option<Uint128>> {
        self.as_ref().get_user_weight(user)
    }
}

impl<'a> WeightsRepositoryMut<'a> for MembershipWeightsRepositoryMut<'a> {
    fn set_total_weight(&mut self, total_weight: Uint128) -> DistributorResult<()> {
        EFFECTIVE_TOTAL_WEIGHT.save(self.deps.storage, &total_weight)?;
        Ok(())
    }

    fn set_user_weight(&mut self, user: Addr, weight: Uint128) -> DistributorResult<Uint128> {
        let minimum_eligible_weight = MINIMUM_ELIGIBLE_WEIGHT.load(self.deps.storage)?;

        USER_WEIGHTS.save(self.deps.storage, user.clone(), &weight)?;

        let effective_user_weight =
            Self::calculate_effective_weight(weight, minimum_eligible_weight);
        EFFECTIVE_USER_WEIGHTS.save(self.deps.storage, user, &effective_user_weight)?;

        Ok(effective_user_weight)
    }
}

impl MembershipWeightsRepositoryMut<'_> {
    fn calculate_effective_weight(weight: Uint128, minimum_eligible_weight: Uint128) -> Uint128 {
        if weight >= minimum_eligible_weight {
            weight
        } else {
            Uint128::zero()
        }
    }
}

///////////////////////////////////
////////// PARTICIPATION //////////
///////////////////////////////////

pub struct ParticipationWeightsRepository<'a> {
    deps: Deps<'a>,
}

impl WeightsRepository for ParticipationWeightsRepository<'_> {
    fn get_total_weight(&self) -> DistributorResult<Uint128> {
        let total_weight = PARTICIPATION_TOTAL_WEIGHT.load(self.deps.storage)?;
        Ok(total_weight)
    }

    // TODO: this should go to interactor, not here
    fn get_user_weight(&self, user: Addr) -> DistributorResult<Option<Uint128>> {
        let components = query_enterprise_components(self.deps)?;

        let tracked_proposal_ids = get_proposal_ids_tracked(self.deps)?;

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
    fn get_total_weight(&self) -> DistributorResult<Uint128> {
        self.as_ref().get_total_weight()
    }

    fn get_user_weight(&self, user: Addr) -> DistributorResult<Option<Uint128>> {
        self.as_ref().get_user_weight(user)
    }
}

impl<'a> WeightsRepositoryMut<'a> for ParticipationWeightsRepositoryMut<'a> {
    fn set_total_weight(&mut self, total_weight: Uint128) -> DistributorResult<()> {
        PARTICIPATION_TOTAL_WEIGHT.save(self.deps.storage, &total_weight)?;
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
        DistributionType::Membership => Box::new(MembershipWeightsRepository { deps }),
        DistributionType::Participation => Box::new(ParticipationWeightsRepository { deps }),
    }
}

pub fn weights_repository_mut<'a>(
    deps: DepsMut<'a>,
    distribution_type: DistributionType,
) -> Box<dyn WeightsRepositoryMut + 'a> {
    match distribution_type {
        DistributionType::Membership => Box::new(MembershipWeightsRepositoryMut { deps }),
        DistributionType::Participation => Box::new(ParticipationWeightsRepositoryMut { deps }),
    }
}
