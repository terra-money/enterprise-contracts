use crate::asset_whitelist::add_whitelisted_assets;
use cosmwasm_std::DepsMut;
use cw_asset::AssetInfo;
use cw_storage_plus::Item;
use enterprise_protocol::error::DaoResult;

const ASSET_WHITELIST: Item<Vec<AssetInfo>> = Item::new("asset_whitelist");

pub fn migrate_asset_whitelist(deps: DepsMut) -> DaoResult<()> {
    let assets = ASSET_WHITELIST.may_load(deps.storage)?.unwrap_or_default();
    ASSET_WHITELIST.remove(deps.storage);

    add_whitelisted_assets(deps, assets)?;

    Ok(())
}
