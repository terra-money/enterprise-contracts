use crate::total_weight::load_total_weight;
use common::cw::QueryContext;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Map, MultiIndex, PrefixBound, UniqueIndex};
use membership_common_api::api::{TotalWeightAboveParams, TotalWeightResponse};
use std::ops::Sub;

#[cw_serde]
pub struct MemberWeight {
    pub user: Addr,
    pub weight: Uint128,
}

pub struct MemberWeightsIndices<'a> {
    pub user: UniqueIndex<'a, Addr, MemberWeight, Addr>,
    pub weight: MultiIndex<'a, u128, MemberWeight, Addr>,
}

impl IndexList<MemberWeight> for MemberWeightsIndices<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<MemberWeight>> + '_> {
        let v: Vec<&dyn Index<MemberWeight>> = vec![&self.user, &self.weight];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn MEMBER_WEIGHTS<'a>() -> IndexedMap<'a, Addr, MemberWeight, MemberWeightsIndices<'a>> {
    let indexes = MemberWeightsIndices {
        user: UniqueIndex::new(
            |member_weight| member_weight.user.clone(),
            "member_weights__user",
        ),
        weight: MultiIndex::new(
            |_, member_weight| member_weight.weight.u128(),
            "member_weights",
            "member_weights__weight",
        ),
    };
    IndexedMap::new("member_weights", indexes)
}

// TODO: migrate this in all types of membership contracts to create the indexed map
pub const MEMBER_WEIGHTS_OLD: Map<Addr, Uint128> = Map::new("membership_common__member_weights");

pub fn get_member_weight(storage: &dyn Storage, member: Addr) -> StdResult<Uint128> {
    Ok(MEMBER_WEIGHTS()
        .may_load(storage, member)?
        .map(|it| it.weight)
        .unwrap_or_default())
}

pub fn set_member_weight(
    storage: &mut dyn Storage,
    member: Addr,
    amount: Uint128,
) -> StdResult<()> {
    MEMBER_WEIGHTS().save(
        storage,
        member.clone(),
        &MemberWeight {
            user: member,
            weight: amount,
        },
    )?;

    Ok(())
}

pub fn increment_member_weight(
    storage: &mut dyn Storage,
    member: Addr,
    amount: Uint128,
) -> StdResult<Uint128> {
    let member_weight = get_member_weight(storage, member.clone())?;
    let new_member_weight = member_weight + amount;
    set_member_weight(storage, member, new_member_weight)?;

    Ok(new_member_weight)
}

pub fn decrement_member_weight(
    storage: &mut dyn Storage,
    member: Addr,
    amount: Uint128,
) -> StdResult<Uint128> {
    let member_weight = get_member_weight(storage, member.clone())?;
    let new_member_weight = member_weight - amount;
    set_member_weight(storage, member, new_member_weight)?;

    Ok(new_member_weight)
}

pub fn total_member_weight_above(
    storage: &dyn Storage,
    above_weight_inclusive: Uint128,
) -> StdResult<Uint128> {
    let total_weight_below: Uint128 = MEMBER_WEIGHTS()
        .idx
        .weight
        .prefix_range(
            storage,
            None,
            Some(PrefixBound::exclusive(above_weight_inclusive)),
            Ascending,
        )
        .map(|res| res.map(|(_, weight)| weight.weight))
        .collect::<StdResult<Vec<Uint128>>>()?
        .into_iter()
        .sum();

    let total_weight = load_total_weight(storage)?;

    let total_weight_above = total_weight.sub(total_weight_below);

    Ok(total_weight_above)
}

pub fn query_total_weight_above(
    qctx: &QueryContext,
    params: TotalWeightAboveParams,
) -> StdResult<TotalWeightResponse> {
    let total_weight = total_member_weight_above(qctx.deps.storage, params.above_weight_inclusive)?;

    Ok(TotalWeightResponse { total_weight })
}
