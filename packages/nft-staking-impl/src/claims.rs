use cosmwasm_std::{Addr, StdResult, Storage, Uint64};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
use nft_staking_api::api::{NftClaim, NftTokenId, ReleaseAt};

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
