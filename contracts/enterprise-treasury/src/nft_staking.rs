use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Index, IndexList, IndexedMap};
use nft_staking_api::api::NftTokenId;

#[cw_serde]
pub struct NftStake {
    pub staker: Addr,
    pub token_id: NftTokenId,
}

pub struct NftStakesIndexes {}

impl IndexList<NftStake> for NftStakesIndexes {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<NftStake>> + '_> {
        let v: Vec<&dyn Index<NftStake>> = vec![];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn NFT_STAKES<'a>() -> IndexedMap<'a, NftTokenId, NftStake, NftStakesIndexes> {
    let indexes = NftStakesIndexes {};
    IndexedMap::new("nft_stakes", indexes)
}
