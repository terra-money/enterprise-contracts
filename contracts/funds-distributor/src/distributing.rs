use crate::state::{CW20_GLOBAL_INDICES, NATIVE_GLOBAL_INDICES};
use crate::state::{EFFECTIVE_TOTAL_WEIGHT, ENTERPRISE_CONTRACT};
use common::cw::Context;
use cosmwasm_std::{Decimal, Response, Uint128};
use cw20::Cw20ReceiveMsg;
use cw_asset::AssetInfo;
use enterprise_protocol::api::ComponentContractsResponse;
use enterprise_protocol::msg::QueryMsg::ComponentContracts;
use enterprise_treasury_api::api::{AssetWhitelistParams, AssetWhitelistResponse};
use enterprise_treasury_api::msg::QueryMsg::AssetWhitelist;
use funds_distributor_api::error::DistributorError::{
    DistributingNonWhitelistedAsset, ZeroTotalWeight,
};
use funds_distributor_api::error::DistributorResult;
use funds_distributor_api::response::{
    cw20_hook_distribute_cw20_response, execute_distribute_native_response,
};
use std::ops::Not;

/// Distributes new rewards for a native asset, using funds found in MessageInfo.
/// Will increase global index for each of the assets being distributed.
pub fn distribute_native(ctx: &mut Context) -> DistributorResult<Response> {
    let funds = ctx.info.funds.clone();

    let distribution_assets = funds
        .iter()
        .map(|coin| AssetInfo::native(coin.denom.to_string()))
        .collect();
    assert_assets_whitelisted(ctx, distribution_assets)?;

    let total_weight = EFFECTIVE_TOTAL_WEIGHT.load(ctx.deps.storage)?;
    if total_weight == Uint128::zero() {
        return Err(ZeroTotalWeight);
    }

    for fund in funds {
        let global_index = NATIVE_GLOBAL_INDICES
            .may_load(ctx.deps.storage, fund.denom.clone())?
            .unwrap_or(Decimal::zero());

        // calculate how many units of the asset we're distributing per unit of total user weight
        // and add that to the global index for the asset
        let index_increment = Decimal::from_ratio(fund.amount, total_weight);

        NATIVE_GLOBAL_INDICES.save(
            ctx.deps.storage,
            fund.denom,
            &global_index.checked_add(index_increment)?,
        )?;
    }

    Ok(execute_distribute_native_response(total_weight))
}

/// Distributes new rewards for a CW20 asset.
/// Will increase global index for the asset being distributed.
pub fn distribute_cw20(ctx: &mut Context, cw20_msg: Cw20ReceiveMsg) -> DistributorResult<Response> {
    let cw20_addr = ctx.info.sender.clone();

    assert_assets_whitelisted(ctx, vec![AssetInfo::cw20(cw20_addr.clone())])?;

    let total_weight = EFFECTIVE_TOTAL_WEIGHT.load(ctx.deps.storage)?;
    if total_weight == Uint128::zero() {
        return Err(ZeroTotalWeight);
    }

    let global_index = CW20_GLOBAL_INDICES
        .may_load(ctx.deps.storage, cw20_addr.clone())?
        .unwrap_or(Decimal::zero());

    // calculate how many units of the asset we're distributing per unit of total user weight
    // and add that to the global index for the asset
    let global_index_increment = Decimal::from_ratio(cw20_msg.amount, total_weight);

    CW20_GLOBAL_INDICES.save(
        ctx.deps.storage,
        cw20_addr.clone(),
        &global_index.checked_add(global_index_increment)?,
    )?;

    Ok(cw20_hook_distribute_cw20_response(
        total_weight,
        cw20_addr.to_string(),
        cw20_msg.amount,
    ))
}

fn assert_assets_whitelisted(ctx: &Context, mut assets: Vec<AssetInfo>) -> DistributorResult<()> {
    let enterprise_components = query_enterprise_components(ctx)?;

    // query asset whitelist with no bounds
    let mut asset_whitelist_response: AssetWhitelistResponse = ctx.deps.querier.query_wasm_smart(
        enterprise_components
            .enterprise_treasury_contract
            .to_string(),
        &AssetWhitelist(AssetWhitelistParams {
            start_after: None,
            limit: None,
        }),
    )?;

    let mut global_asset_whitelist_response: AssetWhitelistResponse =
        ctx.deps.querier.query_wasm_smart(
            enterprise_components
                .enterprise_factory_contract
                .to_string(),
            &enterprise_factory_api::msg::QueryMsg::GlobalAssetWhitelist {},
        )?;

    let mut unified_asset_whitelist = asset_whitelist_response.assets.clone();

    unified_asset_whitelist.append(&mut global_asset_whitelist_response.assets);

    // keep assets that are not found in the whitelist - i.e. remove whitelisted assets
    assets.retain(|asset| unified_asset_whitelist.contains(asset).not());

    // get last asset from the response - will be None iff response is empty
    let mut last_whitelist_asset = asset_whitelist_response.assets.last();

    // repeat until we have either seen all our assets in the whitelist responses, or there
    // are no more assets in the whitelist
    while !assets.is_empty() && last_whitelist_asset.is_some() {
        // now query the whitelist with bounds
        let start_after = last_whitelist_asset.map(|asset| asset.into());
        asset_whitelist_response = ctx.deps.querier.query_wasm_smart(
            enterprise_components
                .enterprise_treasury_contract
                .to_string(),
            &AssetWhitelist(AssetWhitelistParams {
                start_after,
                limit: None,
            }),
        )?;

        // repeat the logic
        assets.retain(|asset| asset_whitelist_response.assets.contains(asset).not());

        last_whitelist_asset = asset_whitelist_response.assets.last();
    }

    if assets.is_empty() {
        Ok(())
    } else {
        Err(DistributingNonWhitelistedAsset)
    }
}

fn query_enterprise_components(ctx: &Context) -> DistributorResult<ComponentContractsResponse> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(ctx.deps.storage)?;

    let component_contracts: ComponentContractsResponse = ctx
        .deps
        .querier
        .query_wasm_smart(enterprise_contract.to_string(), &ComponentContracts {})?;

    Ok(component_contracts)
}
