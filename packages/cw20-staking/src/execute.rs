use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{wasm_execute, BlockInfo, StdResult, SubMsg, Uint128};
use cw_asset::Asset;
use cw_utils::Duration;
use itertools::Itertools;
use std::ops::{Add, Sub};

use crate::api::{Config, Cw20Claim, ReleaseAt, UnstakingResponse};
use crate::claims::{get_and_increment_claims_counter, CLAIMS, CLAIMS_ID_COUNTER};
use crate::error::StakingError::{InsufficientStakedAssets, InvalidStakingAsset, NothingToClaim};
use crate::error::StakingResult;
use crate::state::{CONFIG, STAKES, TOTAL_STAKED};
use common::cw::Context;

// TODO: tests
// TODO: docs
pub fn initialize_staking(ctx: &mut Context, config: Config) -> StakingResult<()> {
    CONFIG.save(ctx.deps.storage, &config)?;
    CLAIMS_ID_COUNTER.save(ctx.deps.storage, &0u64)?;

    Ok(())
}

// TODO: tests
// TODO: docs
pub fn stake_cw20(ctx: &mut Context, amount: Uint128, sender: String) -> StakingResult<Uint128> {
    // only membership CW20 contract can execute this message
    let staking_token = CONFIG.load(ctx.deps.storage)?.cw20_staking_asset;

    if ctx.info.sender != staking_token {
        return Err(InvalidStakingAsset);
    }

    let total_staked = TOTAL_STAKED.may_load(ctx.deps.storage)?.unwrap_or_default();
    let new_total_staked = total_staked.add(amount);
    TOTAL_STAKED.save(ctx.deps.storage, &new_total_staked)?;

    let sender = ctx.deps.api.addr_validate(&sender)?;
    let stake = STAKES
        .may_load(ctx.deps.storage, sender.clone())?
        .unwrap_or_default();

    let new_stake = stake.add(amount);
    STAKES.save(ctx.deps.storage, sender, &new_stake)?;

    Ok(new_stake)
}

// TODO: tests
// TODO: docs
pub fn unstake(ctx: &mut Context, unstake_amount: Uint128) -> StakingResult<UnstakingResponse> {
    let total_staked = TOTAL_STAKED.may_load(ctx.deps.storage)?.unwrap_or_default();
    // TODO: check if there's enough staked assets? this will fail anyway, but maybe a better error msg is in order
    let new_total_staked = total_staked.sub(unstake_amount);
    TOTAL_STAKED.save(ctx.deps.storage, &new_total_staked)?;

    let stake = STAKES
        .may_load(ctx.deps.storage, ctx.info.sender.clone())?
        .unwrap_or_default();

    if stake < unstake_amount {
        return Err(InsufficientStakedAssets);
    }

    let new_stake = stake.sub(unstake_amount);
    STAKES.save(ctx.deps.storage, ctx.info.sender.clone(), &new_stake)?;

    let config = CONFIG.load(ctx.deps.storage)?;
    let unlocking_period = config.unstaking_period;

    let submsgs =
        if unlocking_period == Duration::Time(0) || unlocking_period == Duration::Height(0) {
            vec![SubMsg::new(wasm_execute(
                config.cw20_staking_asset,
                &cw20::Cw20ExecuteMsg::Transfer {
                    recipient: ctx.info.sender.to_string(),
                    amount: unstake_amount,
                },
                vec![],
            )?)]
        } else {
            create_claim(ctx, unlocking_period, unstake_amount)?;

            vec![]
        };

    Ok(UnstakingResponse {
        new_staked_amount: new_stake,
        msgs: submsgs,
    })
}

fn create_claim(
    ctx: &mut Context,
    unlocking_period: Duration,
    unstake_amount: Uint128,
) -> StakingResult<()> {
    let release_at = match unlocking_period {
        Duration::Height(height) => ReleaseAt::Height((ctx.env.block.height + height).into()),
        Duration::Time(time) => ReleaseAt::Timestamp(ctx.env.block.time.plus_seconds(time)),
    };

    let claim_id = get_and_increment_claims_counter(ctx.deps.storage)?;

    CLAIMS().save(
        ctx.deps.storage,
        claim_id,
        &Cw20Claim {
            id: claim_id,
            user: ctx.info.sender.clone(),
            amount: unstake_amount,
            release_at,
        },
    )?;

    Ok(())
}

// TODO: tests
// TODO: docs
pub fn claim(ctx: &mut Context, user: String) -> StakingResult<SubMsg> {
    let user = ctx.deps.api.addr_validate(&user)?;

    let claims = CLAIMS()
        .idx
        .user
        .prefix(user.clone())
        .range(ctx.deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<(u64, Cw20Claim)>>>()?
        .into_iter()
        .map(|(_, claim)| claim)
        .collect_vec();

    if claims.is_empty() {
        return Err(NothingToClaim);
    }

    let block = ctx.env.block.clone();

    let mut releasable_amount: Uint128 = Uint128::zero();
    claims.into_iter().try_for_each(|claim| {
        if is_releasable(&claim, &block) {
            releasable_amount = releasable_amount.add(claim.amount);
            CLAIMS().remove(ctx.deps.storage, claim.id)
        } else {
            Ok(())
        }
    })?;

    let staking_asset = CONFIG.load(ctx.deps.storage)?.cw20_staking_asset;

    let transfer_submsg =
        SubMsg::new(Asset::cw20(staking_asset, releasable_amount).transfer_msg(user)?);

    Ok(transfer_submsg)
}

// TODO: move to something like 'common'? also tests
pub fn is_releasable(claim: &Cw20Claim, block_info: &BlockInfo) -> bool {
    match claim.release_at {
        ReleaseAt::Timestamp(timestamp) => block_info.time >= timestamp,
        ReleaseAt::Height(height) => block_info.height >= height.u64(),
    }
}
