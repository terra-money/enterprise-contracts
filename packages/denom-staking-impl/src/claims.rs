use cosmwasm_std::{Addr, BlockInfo, Order, StdResult, Storage, Uint128, Uint64};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use itertools::Itertools;

use common::cw::ReleaseAt;
use denom_staking_api::api::{ClaimsResponse, DenomClaim};
use denom_staking_api::error::DenomStakingResult;

const CLAIM_IDS: Item<Uint64> = Item::new("claim_ids");

pub struct ClaimsIndexes<'a> {
    pub user: MultiIndex<'a, Addr, DenomClaim, u64>,
}

impl IndexList<DenomClaim> for ClaimsIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<DenomClaim>> + '_> {
        let v: Vec<&dyn Index<DenomClaim>> = vec![&self.user];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn DENOM_CLAIMS<'a>() -> IndexedMap<'a, u64, DenomClaim, ClaimsIndexes<'a>> {
    let indexes = ClaimsIndexes {
        user: MultiIndex::new(
            |_, denom_claim| denom_claim.user.clone(),
            "denom_claims",
            "denom_claims__user",
        ),
    };
    IndexedMap::new("denom_claims", indexes)
}

/// We temporarily store claims that were sent to a foreign chain.
/// This allows us to revert deletion of a user's claim in case the IBC transfer fails.
pub const PENDING_IBC_CLAIMS: Map<u64, Vec<DenomClaim>> = Map::new("pending_ibc_claims");

/// Create and store a new claim.
pub fn add_claim(
    storage: &mut dyn Storage,
    user: Addr,
    amount: Uint128,
    release_at: ReleaseAt,
) -> StdResult<DenomClaim> {
    let next_claim_id = CLAIM_IDS.may_load(storage)?.unwrap_or_default();
    CLAIM_IDS.save(storage, &(next_claim_id + Uint64::one()))?;

    let claim = DenomClaim {
        id: next_claim_id,
        user,
        amount,
        release_at,
    };

    DENOM_CLAIMS().save(storage, next_claim_id.into(), &claim)?;

    Ok(claim)
}

pub fn is_releasable(claim: &DenomClaim, block_info: &BlockInfo) -> bool {
    match claim.release_at {
        ReleaseAt::Timestamp(timestamp) => block_info.time >= timestamp,
        ReleaseAt::Height(height) => block_info.height >= height.u64(),
    }
}

pub fn get_claims(storage: &dyn Storage, user: Addr) -> DenomStakingResult<ClaimsResponse> {
    let claims: Vec<DenomClaim> = DENOM_CLAIMS()
        .idx
        .user
        .prefix(user)
        .range(storage, None, None, Order::Ascending)
        .map_ok(|(_, claim)| claim)
        .collect::<StdResult<Vec<DenomClaim>>>()?;

    Ok(ClaimsResponse { claims })
}

pub fn get_releasable_claims(
    storage: &dyn Storage,
    block: &BlockInfo,
    user: Addr,
) -> DenomStakingResult<ClaimsResponse> {
    let releasable_claims: Vec<DenomClaim> = DENOM_CLAIMS()
        .idx
        .user
        .prefix(user)
        .range(storage, None, None, Order::Ascending)
        .filter_map_ok(|(_, claim)| {
            if is_releasable(&claim, block) {
                Some(claim)
            } else {
                None
            }
        })
        .collect::<StdResult<Vec<DenomClaim>>>()?;

    Ok(ClaimsResponse {
        claims: releasable_claims,
    })
}
