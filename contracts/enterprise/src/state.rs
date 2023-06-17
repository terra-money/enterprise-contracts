use crate::proposals::ProposalInfo;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, StdResult, Storage, Timestamp, Uint128, Uint64};
use cw_storage_plus::{Item, Map};
use enterprise_protocol::api::{
    Claim, ClaimAsset, DaoCouncil, DaoGovConfig, DaoMetadata, DaoType, NftTokenId, ProposalId,
};
use enterprise_protocol::error::DaoResult;

#[cw_serde]
pub struct State {
    pub proposal_being_created: Option<ProposalInfo>,
    pub proposal_being_executed: Option<ProposalId>,
}

pub const STATE: Item<State> = Item::new("state");

pub const DAO_METADATA_KEY: &str = "dao_metadata";

pub const DAO_CREATION_DATE: Item<Timestamp> = Item::new("dao_creation_date");

// TODO: try to unify those below into a single storage structure

// Address of contract which is used to calculate DAO membership
pub const DAO_MEMBERSHIP_CONTRACT: Item<Addr> = Item::new("dao_membership_contract");

pub const ENTERPRISE_FACTORY_CONTRACT: Item<Addr> = Item::new("enterprise_factory_contract");
pub const ENTERPRISE_GOVERNANCE_CONTRACT: Item<Addr> = Item::new("enterprise_governance_contract");
pub const FUNDS_DISTRIBUTOR_CONTRACT: Item<Addr> = Item::new("funds_distributor_contract");

pub const DAO_TYPE: Item<DaoType> = Item::new("dao_type");
pub const DAO_CODE_VERSION: Item<Uint64> = Item::new("dao_code_version");
pub const DAO_METADATA: Item<DaoMetadata> = Item::new(DAO_METADATA_KEY);
pub const DAO_GOV_CONFIG: Item<DaoGovConfig> = Item::new("dao_gov_config");
pub const DAO_COUNCIL: Item<Option<DaoCouncil>> = Item::new("dao_council");

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

pub fn total_cw20_claims(storage: &dyn Storage) -> DaoResult<Uint128> {
    let amount = CLAIMS
        .range(storage, None, None, Ascending)
        .collect::<StdResult<Vec<(Addr, Vec<Claim>)>>>()?
        .into_iter()
        .flat_map(|(_, claims)| claims)
        .fold(Uint128::zero(), |acc, next| {
            if let ClaimAsset::Cw20(asset) = next.asset {
                acc + asset.amount
            } else {
                acc
            }
        });

    Ok(amount)
}

pub fn is_nft_token_id_claimed(storage: &dyn Storage, token_id: NftTokenId) -> DaoResult<bool> {
    let contains_nft_token_id = CLAIMS
        .range(storage, None, None, Ascending)
        .collect::<StdResult<Vec<(Addr, Vec<Claim>)>>>()?
        .into_iter()
        .flat_map(|(_, claims)| claims)
        .any(|claim| {
            if let ClaimAsset::Cw721(asset) = claim.asset {
                asset.tokens.contains(&token_id)
            } else {
                false
            }
        });

    Ok(contains_nft_token_id)
}
