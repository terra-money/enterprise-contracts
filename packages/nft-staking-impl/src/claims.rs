use cosmwasm_std::{Addr, BlockInfo, Order, StdResult, Storage, Uint64};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
use nft_staking_api::api::{ClaimsResponse, NftClaim, NftTokenId, ReleaseAt};
use nft_staking_api::error::NftStakingResult;

const CLAIM_IDS: Item<Uint64> = Item::new("claim_ids");

pub struct ClaimsIndexes<'a> {
    pub user: MultiIndex<'a, Addr, NftClaim, u64>,
}

impl IndexList<NftClaim> for ClaimsIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<NftClaim>> + '_> {
        let v: Vec<&dyn Index<NftClaim>> = vec![&self.user];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn NFT_CLAIMS<'a>() -> IndexedMap<'a, u64, NftClaim, ClaimsIndexes<'a>> {
    let indexes = ClaimsIndexes {
        user: MultiIndex::new(
            |_, nft_claim| nft_claim.user.clone(),
            "nft_claims",
            "nft_claims__user",
        ),
    };
    IndexedMap::new("nft_claims", indexes)
}

/// Create and store a new claim.
pub fn add_claim(
    storage: &mut dyn Storage,
    user: Addr,
    nft_ids: Vec<NftTokenId>,
    release_at: ReleaseAt,
) -> StdResult<NftClaim> {
    let next_claim_id = CLAIM_IDS.may_load(storage)?.unwrap_or_default();
    CLAIM_IDS.save(storage, &(next_claim_id + Uint64::one()))?;

    let claim = NftClaim {
        id: next_claim_id,
        user,
        nft_ids,
        release_at,
    };

    NFT_CLAIMS().save(storage, next_claim_id.into(), &claim)?;

    Ok(claim)
}

pub fn is_releasable(claim: &NftClaim, block_info: &BlockInfo) -> bool {
    match claim.release_at {
        ReleaseAt::Timestamp(timestamp) => block_info.time >= timestamp,
        ReleaseAt::Height(height) => block_info.height >= height.u64(),
    }
}

pub fn get_claims(storage: &dyn Storage, user: Addr) -> NftStakingResult<ClaimsResponse> {
    let claims: Vec<NftClaim> = NFT_CLAIMS()
        .idx
        .user
        .prefix(user)
        .range(storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(u64, NftClaim)>>>()?
        .into_iter()
        .map(|(_, claim)| claim)
        .collect();

    Ok(ClaimsResponse { claims })
}

pub fn get_releasable_claims(
    storage: &dyn Storage,
    block: &BlockInfo,
    user: Addr,
) -> NftStakingResult<ClaimsResponse> {
    let releasable_claims: Vec<NftClaim> = NFT_CLAIMS()
        .idx
        .user
        .prefix(user)
        .range(storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(u64, NftClaim)>>>()?
        .into_iter()
        .filter_map(|(_, claim)| {
            if is_releasable(&claim, block) {
                Some(claim)
            } else {
                None
            }
        })
        .collect();

    Ok(ClaimsResponse {
        claims: releasable_claims,
    })
}
