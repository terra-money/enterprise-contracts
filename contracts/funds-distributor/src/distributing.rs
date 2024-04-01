use crate::asset_types::RewardAsset;
use crate::repository::era_repository::get_current_era;
use crate::repository::global_indices_repository::{
    global_indices_repository, global_indices_repository_mut, GlobalIndicesRepository,
    GlobalIndicesRepositoryMut,
};
use crate::repository::weights_repository::weights_repository;
use crate::state::ENTERPRISE_CONTRACT;
use common::cw::Context;
use cosmwasm_std::{Decimal, Deps, DepsMut, Response, Uint128};
use cw20::Cw20ReceiveMsg;
use cw_asset::AssetInfo;
use enterprise_protocol::api::ComponentContractsResponse;
use enterprise_protocol::msg::QueryMsg::ComponentContracts;
use enterprise_treasury_api::api::{AssetWhitelistParams, AssetWhitelistResponse};
use enterprise_treasury_api::msg::QueryMsg::AssetWhitelist;
use funds_distributor_api::api::DistributionType;
use funds_distributor_api::api::DistributionType::Membership;
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
pub fn distribute_native(
    ctx: &mut Context,
    distribution_type: Option<DistributionType>,
) -> DistributorResult<Response> {
    let funds = ctx.info.funds.clone();

    let assets = funds
        .iter()
        .map(|coin| (RewardAsset::native(coin.denom.clone()), coin.amount))
        .collect();

    let distribution_type = distribution_type.unwrap_or(Membership);
    let total_weight = distribute(ctx.deps.branch(), assets, distribution_type)?;

    Ok(execute_distribute_native_response(total_weight))
}

/// Distributes new rewards for a CW20 asset.
/// Will increase global index for the asset being distributed.
pub fn distribute_cw20(
    ctx: &mut Context,
    cw20_msg: Cw20ReceiveMsg,
    distribution_type: Option<DistributionType>,
) -> DistributorResult<Response> {
    let cw20_addr = ctx.info.sender.clone();

    let assets = vec![(RewardAsset::cw20(cw20_addr.clone()), cw20_msg.amount)];

    let distribution_type = distribution_type.unwrap_or(Membership);
    let total_weight = distribute(ctx.deps.branch(), assets, distribution_type)?;

    Ok(cw20_hook_distribute_cw20_response(
        total_weight,
        cw20_addr.to_string(),
        cw20_msg.amount,
    ))
}

fn distribute(
    mut deps: DepsMut,
    assets: Vec<(RewardAsset, Uint128)>,
    distribution_type: DistributionType,
) -> DistributorResult<Uint128> {
    let distribution_assets = assets.iter().map(|(asset, _)| asset.into()).collect();

    assert_assets_whitelisted(deps.as_ref(), distribution_assets)?;

    let total_weight =
        weights_repository(deps.as_ref(), distribution_type.clone()).get_total_weight()?;
    if total_weight.is_zero() {
        return Err(ZeroTotalWeight);
    }

    let current_era = get_current_era(deps.as_ref())?;

    for (reward_asset, amount) in assets {
        // TODO: what if an asset appears multiple times?
        let global_index = global_indices_repository(deps.as_ref(), distribution_type.clone())
            .get_global_index(reward_asset.clone(), current_era)?
            .unwrap_or(Decimal::zero());

        // calculate how many units of the asset we're distributing per unit of total user weight
        // and add that to the global index for the asset
        let index_increment = Decimal::from_ratio(amount, total_weight);

        let new_global_index = global_index.checked_add(index_increment)?;

        global_indices_repository_mut(deps.branch(), distribution_type.clone()).set_global_index(
            reward_asset,
            new_global_index,
            current_era,
        )?;
    }

    Ok(total_weight)
}

fn assert_assets_whitelisted(deps: Deps, mut assets: Vec<AssetInfo>) -> DistributorResult<()> {
    let enterprise_components = query_enterprise_components(deps)?;

    // query asset whitelist with no bounds
    let mut asset_whitelist_response: AssetWhitelistResponse = deps.querier.query_wasm_smart(
        enterprise_components
            .enterprise_treasury_contract
            .to_string(),
        &AssetWhitelist(AssetWhitelistParams {
            start_after: None,
            limit: None,
        }),
    )?;

    let mut global_asset_whitelist_response: AssetWhitelistResponse =
        deps.querier.query_wasm_smart(
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
        asset_whitelist_response = deps.querier.query_wasm_smart(
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

pub fn query_enterprise_components(deps: Deps) -> DistributorResult<ComponentContractsResponse> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(deps.storage)?;

    let component_contracts: ComponentContractsResponse = deps
        .querier
        .query_wasm_smart(enterprise_contract.to_string(), &ComponentContracts {})?;

    Ok(component_contracts)
}
