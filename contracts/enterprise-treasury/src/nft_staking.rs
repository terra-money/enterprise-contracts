use cosmwasm_schema::cw_serde;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use nft_staking_api::api::NftTokenId;

#[cw_serde]
pub struct NftStake {
    pub staker: Addr,
    pub token_id: NftTokenId,
}

pub struct NftStakesIndexes<'a> {
    pub staker: MultiIndex<'a, Addr, NftStake, NftTokenId>,
}

impl IndexList<NftStake> for NftStakesIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<NftStake>> + '_> {
        let v: Vec<&dyn Index<NftStake>> = vec![&self.staker];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn NFT_STAKES<'a>() -> IndexedMap<'a, NftTokenId, NftStake, NftStakesIndexes<'a>> {
    let indexes = NftStakesIndexes {
        staker: MultiIndex::new(
            |_, nft_stake| nft_stake.staker.clone(),
            "nft_stakes",
            "nft_stakes__staker",
        ),
    };
    IndexedMap::new("nft_stakes", indexes)
}

pub fn load_all_nft_stakes_for_user(store: &dyn Storage, user: Addr) -> StdResult<Option<Uint128>> {
    let nft_stakes = NFT_STAKES()
        .idx
        .staker
        .prefix(user)
        .range(store, None, None, Ascending)
        .count();

    if nft_stakes == 0usize {
        Ok(None)
    } else {
        Ok(Some(Uint128::from(nft_stakes as u128)))
    }
}
