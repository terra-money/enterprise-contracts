use crate::state::{CW20_GLOBAL_INDICES, NATIVE_GLOBAL_INDICES, TOTAL_STAKED};
use common::cw::Context;
use cosmwasm_std::{Decimal, Response, Uint128};
use cw20::Cw20ReceiveMsg;
use funds_distributor_api::error::DistributorError::NothingStaked;
use funds_distributor_api::error::DistributorResult;
use std::ops::Add;

pub fn distribute_native(ctx: &mut Context) -> DistributorResult<Response> {
    let funds = ctx.info.funds.clone();

    let total_staked = TOTAL_STAKED.load(ctx.deps.storage)?;
    if total_staked == Uint128::zero() {
        return Err(NothingStaked);
    }

    for fund in funds {
        let global_index = NATIVE_GLOBAL_INDICES
            .may_load(ctx.deps.storage, fund.denom.clone())?
            .unwrap_or(Decimal::zero());

        let index_increment = Decimal::from_ratio(fund.amount, total_staked);

        NATIVE_GLOBAL_INDICES.save(
            ctx.deps.storage,
            fund.denom,
            &global_index.add(index_increment),
        )?;
    }

    Ok(Response::new()
        .add_attribute("action", "distribute_native")
        .add_attribute("total_staked", total_staked.to_string()))
}

pub fn distribute_cw20(ctx: &mut Context, cw20_msg: Cw20ReceiveMsg) -> DistributorResult<Response> {
    let total_staked = TOTAL_STAKED.load(ctx.deps.storage)?;
    if total_staked == Uint128::zero() {
        return Err(NothingStaked);
    }

    let cw20_asset = ctx.info.sender.clone();

    let global_index = CW20_GLOBAL_INDICES
        .may_load(ctx.deps.storage, cw20_asset.clone())?
        .unwrap_or(Decimal::zero());

    let global_index_increment = Decimal::from_ratio(cw20_msg.amount, total_staked);

    CW20_GLOBAL_INDICES.save(
        ctx.deps.storage,
        cw20_asset.clone(),
        &global_index.add(global_index_increment),
    )?;

    Ok(Response::new()
        .add_attribute("action", "distribute_cw20")
        .add_attribute("total_staked", total_staked.to_string())
        .add_attribute("cw20_asset", cw20_asset.to_string())
        .add_attribute("amount_distributed", cw20_msg.amount.to_string()))
}
