use crate::asset_whitelist::add_whitelisted_assets;
use crate::state::{DAO_MEMBERSHIP_CONTRACT, DAO_TYPE, NFT_WHITELIST};
use cosmwasm_std::DepsMut;
use cw_asset::AssetInfo;
use cw_storage_plus::Item;
use enterprise_protocol::api::DaoType;
use enterprise_protocol::error::DaoResult;
use DaoType::{Multisig, Nft, Token};

const ASSET_WHITELIST: Item<Vec<AssetInfo>> = Item::new("asset_whitelist");

pub fn migrate_asset_whitelist(deps: DepsMut) -> DaoResult<()> {
    let assets = ASSET_WHITELIST.may_load(deps.storage)?.unwrap_or_default();
    ASSET_WHITELIST.remove(deps.storage);

    add_whitelisted_assets(deps, assets)?;

    Ok(())
}

pub fn whitelist_dao_membership_asset(deps: DepsMut) -> DaoResult<()> {
    let dao_type = DAO_TYPE.load(deps.storage)?;
    match dao_type {
        Token => {
            let membership_token = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;
            add_whitelisted_assets(deps, vec![AssetInfo::cw20(membership_token)])?;
        }
        Nft => {
            let membership_nft = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;
            NFT_WHITELIST.save(deps.storage, membership_nft, &())?;
        }
        Multisig => {
            // no-op
        }
    }

    Ok(())
}
