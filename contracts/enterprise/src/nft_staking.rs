use cosmwasm_schema::cw_serde;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use enterprise_protocol::api::NftTokenId;

#[cw_serde]
pub struct NftStake {
    pub staker: Addr,
    pub token_id: NftTokenId,
}

pub struct NftStakesIndexes<'a> {
    pub staker: MultiIndex<'a, Addr, NftStake, String>,
}

impl IndexList<NftStake> for NftStakesIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<NftStake>> + '_> {
        let v: Vec<&dyn Index<NftStake>> = vec![&self.staker];
        Box::new(v.into_iter())
    }
}

#[allow(non_snake_case)]
pub fn NFT_STAKES<'a>() -> IndexedMap<'a, String, NftStake, NftStakesIndexes<'a>> {
    let indexes = NftStakesIndexes {
        staker: MultiIndex::new(
            |_, nft_stake| nft_stake.staker.clone(),
            "nft_stakes",
            "nft_stakes__staker",
        ),
    };
    IndexedMap::new("nft_stakes", indexes)
}

pub fn save_nft_stake(store: &mut dyn Storage, nft_stake: &NftStake) -> StdResult<()> {
    NFT_STAKES().save(store, nft_stake.token_id.clone(), nft_stake)
}

pub fn load_all_nft_stakes_for_user(store: &dyn Storage, user: Addr) -> StdResult<Vec<NftStake>> {
    let nft_stakes = NFT_STAKES()
        .idx
        .staker
        .prefix(user)
        .range(store, None, None, Ascending)
        .collect::<StdResult<Vec<_>>>()?
        .into_iter()
        .map(|(_, stake)| stake)
        .collect();
    Ok(nft_stakes)
}
