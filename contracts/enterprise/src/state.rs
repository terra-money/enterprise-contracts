use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdResult, Storage, Timestamp, Uint64};
use cw_asset::AssetInfo;
use cw_storage_plus::{Item, Map};
use enterprise_protocol::api::{Claim, DaoGovConfig, DaoMetadata, DaoType, ProposalAction};

#[cw_serde]
pub struct State {
    pub multisig_dao_proposal_actions: Option<Vec<ProposalAction>>,
}

pub const STATE: Item<State> = Item::new("state");

pub const DAO_CREATION_DATE: Item<Timestamp> = Item::new("dao_creation_date");

// TODO: try to unify those below into a single storage structure

// Address of contract which is used to calculate DAO membership
pub const DAO_MEMBERSHIP_CONTRACT: Item<Addr> = Item::new("dao_membership_contract");

pub const ENTERPRISE_FACTORY_CONTRACT: Item<Addr> = Item::new("enterprise_factory_contract");

pub const DAO_TYPE: Item<DaoType> = Item::new("dao_type");
pub const DAO_CODE_VERSION: Item<Uint64> = Item::new("dao_code_version");
pub const DAO_METADATA: Item<DaoMetadata> = Item::new("dao_metadata");
pub const DAO_GOV_CONFIG: Item<DaoGovConfig> = Item::new("dao_gov_config");
pub const ASSET_WHITELIST: Item<Vec<AssetInfo>> = Item::new("asset_whitelist");
pub const NFT_WHITELIST: Map<Addr, ()> = Map::new("nft_whitelist");

// TODO: use indexed map and then add pagination to the queries
pub const CLAIMS: Map<&Addr, Vec<Claim>> = Map::new("claims");

pub fn add_claim(storage: &mut dyn Storage, addr: &Addr, claim: Claim) -> StdResult<()> {
    CLAIMS.update(storage, addr, |claims| -> StdResult<Vec<Claim>> {
        let mut claims = claims.unwrap_or_default();
        claims.push(claim);
        Ok(claims)
    })?;
    Ok(())
}
