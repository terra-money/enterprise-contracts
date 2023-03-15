use crate::state::TOTAL_WEIGHT;
use crate::state::{CW20_GLOBAL_INDICES, NATIVE_GLOBAL_INDICES};
use common::cw::Context;
use cosmwasm_std::{Decimal, Response, Uint128};
use cw20::Cw20ReceiveMsg;
use funds_distributor_api::error::DistributorError::ZeroTotalWeight;
use funds_distributor_api::error::DistributorResult;
use std::ops::Add;

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

        let index_increment = Decimal::from_ratio(fund.amount, total_weight);

        NATIVE_GLOBAL_INDICES.save(
            ctx.deps.storage,
            fund.denom,
            &global_index.add(index_increment),
        )?;
    }

    Ok(Response::new()
        .add_attribute("action", "distribute_native")
        .add_attribute("total_weight", total_weight.to_string()))
}

pub fn distribute_cw20(ctx: &mut Context, cw20_msg: Cw20ReceiveMsg) -> DistributorResult<Response> {
    let total_weight = TOTAL_WEIGHT.load(ctx.deps.storage)?;
    if total_weight == Uint128::zero() {
        return Err(ZeroTotalWeight);
    }

    let cw20_asset = ctx.info.sender.clone();

    let global_index = CW20_GLOBAL_INDICES
        .may_load(ctx.deps.storage, cw20_asset.clone())?
        .unwrap_or(Decimal::zero());

    let global_index_increment = Decimal::from_ratio(cw20_msg.amount, total_weight);

    CW20_GLOBAL_INDICES.save(
        ctx.deps.storage,
        cw20_asset.clone(),
        &global_index.add(global_index_increment),
    )?;

    Ok(Response::new()
        .add_attribute("action", "distribute_cw20")
        .add_attribute("total_weight", total_weight.to_string())
        .add_attribute("cw20_asset", cw20_asset.to_string())
        .add_attribute("amount_distributed", cw20_msg.amount.to_string()))
}
