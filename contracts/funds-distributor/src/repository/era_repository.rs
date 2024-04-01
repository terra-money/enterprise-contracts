use crate::state::EraId;
use cosmwasm_std::{Addr, Deps, DepsMut, StdResult};
use cw_storage_plus::{Item, Map};
use funds_distributor_api::api::DistributionType;
use funds_distributor_api::error::DistributorResult;
use DistributionType::{Membership, Participation};

pub const FIRST_ERA: EraId = 1;

const CURRENT_ERA_ID: Item<EraId> = Item::new("current_era_id");

const USER_LAST_CLAIMED_ERA: Map<Addr, EraId> = Map::new("user_last_claimed_era");

const USER_LAST_RESOLVED_ERA: Map<Addr, EraId> = Map::new("user_last_resolved_era");

/// The first era in which this user had a weight.
const MEMBERSHIP_USER_FIRST_ERA_WITH_WEIGHT: Map<Addr, EraId> =
    Map::new("membership_user_first_era_with_weight");
const PARTICIPATION_USER_FIRST_ERA_WITH_WEIGHT: Map<Addr, EraId> =
    Map::new("participation_user_first_era_with_weight");

pub fn get_current_era(deps: Deps) -> DistributorResult<EraId> {
    let era_id = CURRENT_ERA_ID.load(deps.storage)?;
    Ok(era_id)
}

pub fn set_current_era(deps: DepsMut, era_id: EraId) -> DistributorResult<()> {
    CURRENT_ERA_ID.save(deps.storage, &era_id)?;

    Ok(())
}

pub fn increment_era(deps: DepsMut) -> DistributorResult<()> {
    CURRENT_ERA_ID.update(deps.storage, |era| -> StdResult<EraId> { Ok(era + 1) })?;

    Ok(())
}

pub fn get_user_last_fully_claimed_era(deps: Deps, user: Addr) -> DistributorResult<Option<EraId>> {
    let era = USER_LAST_CLAIMED_ERA.may_load(deps.storage, user)?;
    Ok(era)
}

pub fn set_user_last_claimed_era(
    deps: DepsMut,
    user: Addr,
    era_id: EraId,
) -> DistributorResult<()> {
    USER_LAST_CLAIMED_ERA.save(deps.storage, user, &era_id)?;
    Ok(())
}

pub fn get_user_last_resolved_era(deps: Deps, user: Addr) -> DistributorResult<Option<EraId>> {
    let era = USER_LAST_RESOLVED_ERA.may_load(deps.storage, user)?;
    Ok(era)
}

// TODO: do we have to set this anywhere other than 'update user indices'?
pub fn set_user_last_resolved_era(
    deps: DepsMut,
    user: Addr,
    era_id: EraId,
) -> DistributorResult<()> {
    USER_LAST_RESOLVED_ERA.save(deps.storage, user, &era_id)?;
    Ok(())
}

pub fn get_user_first_era_with_weight(
    deps: Deps,
    user: Addr,
    distribution_type: DistributionType,
) -> DistributorResult<Option<EraId>> {
    let era = user_first_era_with_weight_map(distribution_type).may_load(deps.storage, user)?;
    Ok(era)
}

// TODO: set this in 'user weight changed' and wherever else appropriate (user votes changed and such)
pub fn set_user_first_era_with_weight_if_empty(
    deps: DepsMut,
    user: Addr,
    era_id: EraId,
    distribution_type: DistributionType,
) -> DistributorResult<()> {
    let map = user_first_era_with_weight_map(distribution_type);
    let first_era_with_weight = map.may_load(deps.storage, user.clone())?;

    if first_era_with_weight.is_none() {
        map.save(deps.storage, user, &era_id)?;
    }

    Ok(())
}

fn user_first_era_with_weight_map(
    distribution_type: DistributionType,
) -> Map<'static, Addr, EraId> {
    match distribution_type {
        Membership => MEMBERSHIP_USER_FIRST_ERA_WITH_WEIGHT,
        Participation => PARTICIPATION_USER_FIRST_ERA_WITH_WEIGHT,
    }
}
