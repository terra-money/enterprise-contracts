use crate::state::EraId;
use cosmwasm_std::{Addr, Deps, DepsMut};
use cw_storage_plus::{Item, Map};
use funds_distributor_api::error::DistributorResult;

pub const FIRST_ERA: EraId = 1;

const CURRENT_ERA_ID: Item<EraId> = Item::new("current_era_id");

const USER_LAST_CLAIMED_ERA: Map<Addr, EraId> = Map::new("user_last_claimed_era");

const USER_LAST_RESOLVED_ERA: Map<Addr, EraId> = Map::new("user_last_resolved_era");

/// The first era in which this user had a weight.
const USER_FIRST_ERA_WITH_WEIGHT: Map<Addr, EraId> = Map::new("user_first_era_with_weight");

pub fn get_current_era(deps: Deps) -> DistributorResult<EraId> {
    let era_id = CURRENT_ERA_ID.load(deps.storage)?;
    Ok(era_id)
}

pub fn set_current_era(deps: DepsMut, era_id: EraId) -> DistributorResult<()> {
    CURRENT_ERA_ID.save(deps.storage, &era_id)?;

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

pub fn get_user_first_era_with_weight(deps: Deps, user: Addr) -> DistributorResult<Option<EraId>> {
    let era = USER_FIRST_ERA_WITH_WEIGHT.may_load(deps.storage, user)?;
    Ok(era)
}

// TODO: set this in 'user weight changed' and wherever else appropriate (user votes changed and such)
pub fn set_user_first_era_with_weight(
    deps: DepsMut,
    user: Addr,
    era_id: EraId,
) -> DistributorResult<()> {
    USER_FIRST_ERA_WITH_WEIGHT.save(deps.storage, user, &era_id)?;
    Ok(())
}
