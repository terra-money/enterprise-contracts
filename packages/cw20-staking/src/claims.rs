use crate::api::Cw20Claim;
use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};

pub const CLAIMS_ID_COUNTER: Item<u64> = Item::new("cw20_staking:claims_counter");

pub fn get_and_increment_claims_counter(store: &mut dyn Storage) -> StdResult<u64> {
    let claim_id = CLAIMS_ID_COUNTER.load(store)?;
    CLAIMS_ID_COUNTER.save(store, &(claim_id + 1))?;
    Ok(claim_id)
}

pub fn add_claim(storage: &mut dyn Storage, claim: Cw20Claim) -> StdResult<()> {
    CLAIMS().save(storage, claim.id, &claim)?;
    Ok(())
}

pub struct ClaimIndexes<'a> {
    pub user: MultiIndex<'a, Addr, Cw20Claim, u64>,
}

impl IndexList<Cw20Claim> for ClaimIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Cw20Claim>> + '_> {
        let v: Vec<&dyn Index<Cw20Claim>> = vec![&self.user];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn CLAIMS<'a>() -> IndexedMap<'a, u64, Cw20Claim, ClaimIndexes<'a>> {
    let indexes = ClaimIndexes {
        user: MultiIndex::new(|_, claim| claim.user.clone(), "claims", "claims__user"),
    };
    IndexedMap::new("claims", indexes)
}
