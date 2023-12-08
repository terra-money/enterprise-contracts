use cosmwasm_std::{Addr, BlockInfo, Order, StdResult, Storage, Uint128, Uint64};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use itertools::Itertools;

use common::cw::ReleaseAt;
use token_staking_api::api::{ClaimsResponse, TokenClaim};
use token_staking_api::error::TokenStakingResult;

const CLAIM_IDS: Item<Uint64> = Item::new("claim_ids");

pub struct ClaimsIndexes<'a> {
    pub user: MultiIndex<'a, Addr, TokenClaim, u64>,
}

impl IndexList<TokenClaim> for ClaimsIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TokenClaim>> + '_> {
        let v: Vec<&dyn Index<TokenClaim>> = vec![&self.user];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn TOKEN_CLAIMS<'a>() -> IndexedMap<'a, u64, TokenClaim, ClaimsIndexes<'a>> {
    let indexes = ClaimsIndexes {
        user: MultiIndex::new(
            |_, token_claim| token_claim.user.clone(),
            "token_claims",
            "token_claims__user",
        ),
    };
    IndexedMap::new("token_claims", indexes)
}

/// We temporarily store claims that were sent to a foreign chain.
/// This allows us to revert deletion of a user's claim in case the IBC transfer fails.
pub const PENDING_IBC_CLAIMS: Map<u64, Vec<TokenClaim>> = Map::new("pending_ibc_claims");

/// Create and store a new claim.
pub fn add_claim(
    storage: &mut dyn Storage,
    user: Addr,
    amount: Uint128,
    release_at: ReleaseAt,
) -> StdResult<TokenClaim> {
    let next_claim_id = CLAIM_IDS.may_load(storage)?.unwrap_or_default();
    CLAIM_IDS.save(storage, &(next_claim_id + Uint64::one()))?;

    let claim = TokenClaim {
        id: next_claim_id,
        user,
        amount,
        release_at,
    };

    TOKEN_CLAIMS().save(storage, next_claim_id.into(), &claim)?;

    Ok(claim)
}

pub fn is_releasable(claim: &TokenClaim, block_info: &BlockInfo) -> bool {
    match claim.release_at {
        ReleaseAt::Timestamp(timestamp) => block_info.time >= timestamp,
        ReleaseAt::Height(height) => block_info.height >= height.u64(),
    }
}

pub fn get_claims(storage: &dyn Storage, user: Addr) -> TokenStakingResult<ClaimsResponse> {
    let claims: Vec<TokenClaim> = TOKEN_CLAIMS()
        .idx
        .user
        .prefix(user)
        .range(storage, None, None, Order::Ascending)
        .map_ok(|(_, claim)| claim)
        .collect::<StdResult<Vec<TokenClaim>>>()?;

    Ok(ClaimsResponse { claims })
}

pub fn get_releasable_claims(
    storage: &dyn Storage,
    block: &BlockInfo,
    user: Addr,
) -> TokenStakingResult<ClaimsResponse> {
    let releasable_claims: Vec<TokenClaim> = TOKEN_CLAIMS()
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
        .collect::<StdResult<Vec<TokenClaim>>>()?;

    Ok(ClaimsResponse {
        claims: releasable_claims,
    })
}
