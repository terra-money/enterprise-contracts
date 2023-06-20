use common::cw::QueryContext;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, DepsMut, StdError, StdResult};
use cw_asset::AssetInfo;
use cw_storage_plus::{Bound, Map};
use enterprise_protocol::error::DaoResult;

pub const NATIVE_ASSET_WHITELIST: Map<String, ()> = Map::new("native_asset_whitelist");
pub const CW20_ASSET_WHITELIST: Map<Addr, ()> = Map::new("cw20_asset_whitelist");
pub const CW1155_ASSET_WHITELIST: Map<(Addr, String), ()> = Map::new("cw1155_asset_whitelist");

pub fn add_whitelisted_assets(deps: DepsMut, assets: Vec<AssetInfo>) -> DaoResult<()> {
    for asset in assets {
        match asset {
            AssetInfo::Native(denom) => NATIVE_ASSET_WHITELIST.save(deps.storage, denom, &())?,
            AssetInfo::Cw20(addr) => {
                let addr = deps.api.addr_validate(addr.as_ref())?;
                CW20_ASSET_WHITELIST.save(deps.storage, addr, &())?;
            }
            AssetInfo::Cw1155(addr, id) => {
                let addr = deps.api.addr_validate(addr.as_ref())?;
                CW1155_ASSET_WHITELIST.save(deps.storage, (addr, id), &())?;
            }
            _ => return Err(StdError::generic_err("unknown asset type").into()),
        }
    }

    Ok(())
}

pub fn remove_whitelisted_assets(deps: DepsMut, assets: Vec<AssetInfo>) -> DaoResult<()> {
    for asset in assets {
        match asset {
            AssetInfo::Native(denom) => NATIVE_ASSET_WHITELIST.remove(deps.storage, denom),
            AssetInfo::Cw20(addr) => {
                let addr = deps.api.addr_validate(addr.as_ref())?;
                CW20_ASSET_WHITELIST.remove(deps.storage, addr);
            }
            AssetInfo::Cw1155(addr, id) => {
                let addr = deps.api.addr_validate(addr.as_ref())?;
                CW1155_ASSET_WHITELIST.remove(deps.storage, (addr, id));
            }
            _ => return Err(StdError::generic_err("unknown asset type").into()),
        }
    }

    Ok(())
}

pub fn get_whitelisted_assets_starting_with_native(
    qctx: QueryContext,
    start_after: Option<String>,
    limit: usize,
) -> DaoResult<Vec<AssetInfo>> {
    let start_after = start_after.map(Bound::exclusive);

    let mut native_assets: Vec<AssetInfo> = NATIVE_ASSET_WHITELIST
        .range(qctx.deps.storage, start_after, None, Ascending)
        .take(limit)
        .collect::<StdResult<Vec<(String, ())>>>()?
        .into_iter()
        .map(|(denom, _)| AssetInfo::native(denom))
        .collect();

    let native_assets_length = native_assets.len();

    if native_assets_length < limit {
        let mut more_assets =
            get_whitelisted_assets_starting_with_cw20(qctx, None, limit - native_assets_length)?;

        native_assets.append(&mut more_assets);
    };

    Ok(native_assets)
}

pub fn get_whitelisted_assets_starting_with_cw20(
    qctx: QueryContext,
    start_after: Option<Addr>,
    limit: usize,
) -> DaoResult<Vec<AssetInfo>> {
    let start_after = start_after.map(Bound::exclusive);

    let mut cw20_assets: Vec<AssetInfo> = CW20_ASSET_WHITELIST
        .range(qctx.deps.storage, start_after, None, Ascending)
        .take(limit)
        .collect::<StdResult<Vec<(Addr, ())>>>()?
        .into_iter()
        .map(|(addr, _)| AssetInfo::cw20(addr))
        .collect();

    let cw20_assets_length = cw20_assets.len();

    if cw20_assets_length < limit {
        let mut more_assets =
            get_whitelisted_assets_starting_with_cw1155(qctx, None, limit - cw20_assets_length)?;

        cw20_assets.append(&mut more_assets);
    };

    Ok(cw20_assets)
}

pub fn get_whitelisted_assets_starting_with_cw1155(
    qctx: QueryContext,
    start_after: Option<(Addr, String)>,
    limit: usize,
) -> DaoResult<Vec<AssetInfo>> {
    let start_after = start_after.map(Bound::exclusive);

    let cw1155_assets = CW1155_ASSET_WHITELIST
        .range(qctx.deps.storage, start_after, None, Ascending)
        .take(limit)
        .collect::<StdResult<Vec<((Addr, String), ())>>>()?
        .into_iter()
        .map(|((addr, id), _)| AssetInfo::cw1155(addr, id))
        .collect();

    Ok(cw1155_assets)
}
