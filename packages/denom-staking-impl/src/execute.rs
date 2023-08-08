use crate::claims::{add_claim, get_releasable_claims, DENOM_CLAIMS};
use crate::config::CONFIG;
use common::cw::{Context, ReleaseAt};
use cosmwasm_std::{coins, BankMsg, Response, SubMsg, Uint128};
use cw_utils::Duration::{Height, Time};
use denom_staking_api::api::{ClaimMsg, UnstakeMsg, UpdateUnlockingPeriodMsg};
use denom_staking_api::error::DenomStakingError::{
    InsufficientStake, InvalidStakingDenom, MultipleDenomsBeingStaked,
};
use denom_staking_api::error::DenomStakingResult;
use membership_common::admin::admin_caller_only;
use membership_common::member_weights::{
    decrement_member_weight, get_member_weight, increment_member_weight,
};
use membership_common::total_weight::{decrement_total_weight, increment_total_weight};
use membership_common::weight_change_hooks::report_weight_change_submsgs;
use membership_common_api::api::UserWeightChange;

pub fn stake_denom(ctx: &mut Context, user: Option<String>) -> DenomStakingResult<Response> {
    if ctx.info.funds.len() != 1 {
        return Err(MultipleDenomsBeingStaked);
    }

    let coin = ctx.info.funds.first().unwrap();

    let config = CONFIG.load(ctx.deps.storage)?;

    if coin.denom != config.denom {
        return Err(InvalidStakingDenom);
    }

    let user = user
        .map(|user| ctx.deps.api.addr_validate(&user))
        .transpose()?
        .unwrap_or_else(|| ctx.info.sender.clone());

    let old_weight = get_member_weight(ctx.deps.storage, user.clone())?;
    let new_weight = increment_member_weight(ctx.deps.storage, user.clone(), coin.amount)?;
    let new_total_staked = increment_total_weight(ctx, coin.amount)?;

    let report_weight_change_submsgs = report_weight_change_submsgs(
        ctx,
        vec![UserWeightChange {
            user: user.to_string(),
            old_weight,
            new_weight,
        }],
    )?;

    Ok(Response::new()
        .add_attribute("action", "stake")
        .add_attribute("user_stake", new_weight.to_string())
        .add_attribute("total_staked", new_total_staked.to_string())
        .add_submessages(report_weight_change_submsgs))
}

/// Unstake denoms. Only admin can perform this on behalf of a user.
pub fn unstake(ctx: &mut Context, msg: UnstakeMsg) -> DenomStakingResult<Response> {
    // only admin can execute this
    admin_caller_only(ctx)?;

    let user = ctx.deps.api.addr_validate(&msg.user)?;

    let user_stake = get_member_weight(ctx.deps.storage, user.clone())?;

    if user_stake < msg.amount {
        return Err(InsufficientStake);
    }

    let unstaked_amount = msg.amount;

    let new_weight = decrement_member_weight(ctx.deps.storage, user.clone(), unstaked_amount)?;
    let new_total_staked = decrement_total_weight(ctx, unstaked_amount)?;

    let release_at = calculate_release_at(ctx)?;

    let claim = add_claim(ctx.deps.storage, user.clone(), unstaked_amount, release_at)?;

    let report_weight_change_submsgs = report_weight_change_submsgs(
        ctx,
        vec![UserWeightChange {
            user: user.to_string(),
            old_weight: user_stake,
            new_weight,
        }],
    )?;

    Ok(Response::new()
        .add_attribute("action", "unstake")
        .add_attribute("total_staked", new_total_staked.to_string())
        .add_attribute("claim_id", claim.id.to_string())
        .add_submessages(report_weight_change_submsgs))
}

// TODO: move to common?
fn calculate_release_at(ctx: &mut Context) -> DenomStakingResult<ReleaseAt> {
    let config = CONFIG.load(ctx.deps.storage)?;

    let release_at = match config.unlocking_period {
        Height(height) => ReleaseAt::Height((ctx.env.block.height + height).into()),
        Time(time) => ReleaseAt::Timestamp(ctx.env.block.time.plus_seconds(time)),
    };
    Ok(release_at)
}

/// Update the unlocking period. Only the current admin can execute this.
pub fn update_unlocking_period(
    ctx: &mut Context,
    msg: UpdateUnlockingPeriodMsg,
) -> DenomStakingResult<Response> {
    // only admin can execute this
    admin_caller_only(ctx)?;

    let mut config = CONFIG.load(ctx.deps.storage)?;

    if let Some(new_unlocking_period) = msg.new_unlocking_period {
        config.unlocking_period = new_unlocking_period;
    }

    CONFIG.save(ctx.deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_unlocking_period"))
}

/// Claim any unstaked denoms that are ready to be released.
pub fn claim(ctx: &mut Context, msg: ClaimMsg) -> DenomStakingResult<Response> {
    let user = ctx.deps.api.addr_validate(&msg.user)?;

    let releasable_claims =
        get_releasable_claims(ctx.deps.storage, &ctx.env.block, user.clone())?.claims;

    let denom = CONFIG.load(ctx.deps.storage)?.denom;

    let mut claim_amount = Uint128::zero();

    for claim in releasable_claims {
        claim_amount += claim.amount;

        DENOM_CLAIMS().remove(ctx.deps.storage, claim.id.u64())?;
    }

    let send_denoms_submsg = SubMsg::new(BankMsg::Send {
        to_address: user.to_string(),
        amount: coins(claim_amount.u128(), denom),
    });

    Ok(Response::new()
        .add_attribute("action", "claim")
        .add_submessage(send_denoms_submsg))
}
