use cosmwasm_std::{Addr, Deps, DepsMut};
use cw_storage_plus::{Item, Map};
use funds_distributor_api::api::DistributionType;
use funds_distributor_api::error::DistributorResult;
use DistributionType::{Membership, Participation};

pub type EraId = u64;

pub const FIRST_ERA: EraId = 1;

const MEMBERSHIP_CURRENT_ERA_ID: Item<EraId> = Item::new("membership_current_era_id");
const PARTICIPATION_CURRENT_ERA_ID: Item<EraId> = Item::new("participation_current_era_id");

fn current_era_item(distribution_type: DistributionType) -> Item<'static, EraId> {
    match distribution_type {
        Membership => MEMBERSHIP_CURRENT_ERA_ID,
        Participation => PARTICIPATION_CURRENT_ERA_ID,
    }
}

const MEMBERSHIP_USER_LAST_CLAIMED_ERA: Map<Addr, EraId> =
    Map::new("membership_user_last_claimed_era");
const PARTICIPATION_USER_LAST_CLAIMED_ERA: Map<Addr, EraId> =
    Map::new("participation_user_last_claimed_era");

fn user_last_claimed_era_map(distribution_type: DistributionType) -> Map<'static, Addr, EraId> {
    match distribution_type {
        Membership => MEMBERSHIP_USER_LAST_CLAIMED_ERA,
        Participation => PARTICIPATION_USER_LAST_CLAIMED_ERA,
    }
}

const MEMBERSHIP_USER_LAST_RESOLVED_ERA: Map<Addr, EraId> =
    Map::new("membership_user_last_resolved_era");
const PARTICIPATION_USER_LAST_RESOLVED_ERA: Map<Addr, EraId> =
    Map::new("participation_user_last_resolved_era");

fn user_last_resolved_era_map(distribution_type: DistributionType) -> Map<'static, Addr, EraId> {
    match distribution_type {
        Membership => MEMBERSHIP_USER_LAST_RESOLVED_ERA,
        Participation => PARTICIPATION_USER_LAST_RESOLVED_ERA,
    }
}

/// The first era in which this user had a weight.
const MEMBERSHIP_USER_FIRST_ERA_WITH_WEIGHT: Map<Addr, EraId> =
    Map::new("membership_user_first_era_with_weight");
const PARTICIPATION_USER_FIRST_ERA_WITH_WEIGHT: Map<Addr, EraId> =
    Map::new("participation_user_first_era_with_weight");

pub fn get_current_era(
    deps: Deps,
    distribution_type: DistributionType,
) -> DistributorResult<EraId> {
    let era_id = current_era_item(distribution_type).load(deps.storage)?;
    Ok(era_id)
}

pub fn set_current_era(
    deps: DepsMut,
    era_id: EraId,
    distribution_type: DistributionType,
) -> DistributorResult<()> {
    current_era_item(distribution_type).save(deps.storage, &era_id)?;

    Ok(())
}

pub fn get_user_last_fully_claimed_era(
    deps: Deps,
    user: Addr,
    distribution_type: DistributionType,
) -> DistributorResult<Option<EraId>> {
    let era = user_last_claimed_era_map(distribution_type).may_load(deps.storage, user)?;
    Ok(era)
}

pub fn set_user_last_claimed_era(
    deps: DepsMut,
    user: Addr,
    era_id: EraId,
    distribution_type: DistributionType,
) -> DistributorResult<()> {
    user_last_claimed_era_map(distribution_type).save(deps.storage, user, &era_id)?;
    Ok(())
}

pub fn get_user_last_resolved_era(
    deps: Deps,
    user: Addr,
    distribution_type: DistributionType,
) -> DistributorResult<Option<EraId>> {
    let era = user_last_resolved_era_map(distribution_type).may_load(deps.storage, user)?;
    Ok(era)
}

// TODO: do we have to set this anywhere other than 'update user indices'?
pub fn set_user_last_resolved_era(
    deps: DepsMut,
    user: Addr,
    era_id: EraId,
    distribution_type: DistributionType,
) -> DistributorResult<()> {
    user_last_resolved_era_map(distribution_type).save(deps.storage, user, &era_id)?;
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
