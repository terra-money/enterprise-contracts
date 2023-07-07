use crate::state::TOTAL_WEIGHT;
use crate::state::{CW20_GLOBAL_INDICES, NATIVE_GLOBAL_INDICES};
use common::cw::Context;
use cosmwasm_std::{Decimal, Response, Uint128};
use cw20::Cw20ReceiveMsg;
use funds_distributor_api::error::DistributorError::ZeroTotalWeight;
use funds_distributor_api::error::DistributorResult;
use funds_distributor_api::response::{
    cw20_hook_distribute_cw20_response, execute_distribute_native_response,
};
use std::ops::Add;

/// Distributes new rewards for a native asset, using funds found in MessageInfo.
/// Will increase global index for each of the assets being distributed.
pub fn distribute_native(ctx: &mut Context) -> DistributorResult<Response> {
    let funds = ctx.info.funds.clone();

    let total_weight = TOTAL_WEIGHT.load(ctx.deps.storage)?;
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
            &global_index.add(index_increment),
        )?;
    }

    Ok(execute_distribute_native_response(total_weight))
}

/// Distributes new rewards for a CW20 asset.
/// Will increase global index for the asset being distributed.
pub fn distribute_cw20(ctx: &mut Context, cw20_msg: Cw20ReceiveMsg) -> DistributorResult<Response> {
    let total_weight = TOTAL_WEIGHT.load(ctx.deps.storage)?;
    if total_weight == Uint128::zero() {
        return Err(ZeroTotalWeight);
    }

    let cw20_asset = ctx.info.sender.clone();

    let global_index = CW20_GLOBAL_INDICES
        .may_load(ctx.deps.storage, cw20_asset.clone())?
        .unwrap_or(Decimal::zero());

    // calculate how many units of the asset we're distributing per unit of total user weight
    // and add that to the global index for the asset
    let global_index_increment = Decimal::from_ratio(cw20_msg.amount, total_weight);

    CW20_GLOBAL_INDICES.save(
        ctx.deps.storage,
        cw20_asset.clone(),
        &global_index.add(global_index_increment),
    )?;

    Ok(cw20_hook_distribute_cw20_response(
        total_weight,
        cw20_asset.to_string(),
        cw20_msg.amount,
    ))
}
