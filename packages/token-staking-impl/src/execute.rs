use crate::claims::{add_claim, get_releasable_claims, TOKEN_CLAIMS};
use crate::config::CONFIG;
use crate::token_staking::{
    decrement_user_total_staked, get_user_stake, increment_user_stake, set_user_stake,
};
use crate::total_staked::{
    decrement_total_staked, increment_total_staked, load_total_staked, save_total_staked,
};
use crate::validate::admin_caller_only;
use common::cw::Context;
use cosmwasm_std::{from_binary, wasm_execute, Response, StdError, SubMsg, Uint128};
use cw20::Cw20ReceiveMsg;
use cw_utils::Duration::{Height, Time};
use std::ops::Not;
use token_staking_api::api::{ClaimMsg, ReleaseAt, UnstakeMsg, UpdateConfigMsg, UserStake};
use token_staking_api::error::TokenStakingError::{
    IncorrectStakesInitializationAmount, InsufficientStake, StakesAlreadyInitialized, Unauthorized,
};
use token_staking_api::error::TokenStakingResult;
use token_staking_api::msg::Cw20HookMsg;

/// Function to execute when receiving a Receive callback from a CW20 contract.
pub fn receive_cw20(ctx: &mut Context, msg: Cw20ReceiveMsg) -> TokenStakingResult<Response> {
    let config = CONFIG.load(ctx.deps.storage)?;

    // only designated token contract can invoke this
    if ctx.info.sender != config.token_contract {
        return Err(Unauthorized);
    }

    // only admin should send the actual tokens, they'll tell us which user
    if msg.sender != config.admin {
        return Err(Unauthorized);
    }

    match from_binary(&msg.msg) {
        Ok(Cw20HookMsg::Stake { user }) => stake_token(ctx, msg, user),
        Ok(Cw20HookMsg::InitializeStakers { stakers }) => initialize_stakers(ctx, msg, stakers),
        Ok(Cw20HookMsg::AddClaim { user, release_at }) => {
            add_token_claim(ctx, msg, user, release_at)
        }
        _ => Err(StdError::generic_err("msg payload not recognized").into()),
    }
}

fn stake_token(
    ctx: &mut Context,
    msg: Cw20ReceiveMsg,
    user: String,
) -> TokenStakingResult<Response> {
    let user = ctx.deps.api.addr_validate(&user)?;

    let new_user_stake = increment_user_stake(ctx.deps.storage, user, msg.amount)?;
    let new_total_staked = increment_total_staked(ctx, msg.amount)?;

    Ok(Response::new()
        .add_attribute("action", "stake")
        .add_attribute("user_stake", new_user_stake.to_string())
        .add_attribute("total_staked", new_total_staked.to_string()))
}

fn initialize_stakers(
    ctx: &mut Context,
    msg: Cw20ReceiveMsg,
    stakers: Vec<UserStake>,
) -> TokenStakingResult<Response> {
    let total_staked = load_total_staked(ctx.deps.storage)?;

    if total_staked.is_zero().not() {
        return Err(StakesAlreadyInitialized);
    }

    let mut user_stakes_sum = Uint128::zero();

    for staker in stakers {
        let user = ctx.deps.api.addr_validate(&staker.user)?;

        user_stakes_sum += staker.staked_amount;

        set_user_stake(ctx.deps.storage, user, staker.staked_amount)?;
    }

    if user_stakes_sum != msg.amount {
        return Err(IncorrectStakesInitializationAmount);
    }

    save_total_staked(ctx.deps.storage, &user_stakes_sum, &ctx.env.block)?;

    Ok(Response::new()
        .add_attribute("action", "initialize_stakes")
        .add_attribute("total_staked", user_stakes_sum.to_string()))
}

fn add_token_claim(
    ctx: &mut Context,
    msg: Cw20ReceiveMsg,
    user: String,
    release_at: ReleaseAt,
) -> TokenStakingResult<Response> {
    let user = ctx.deps.api.addr_validate(&user)?;

    let claim = add_claim(ctx.deps.storage, user, msg.amount, release_at)?;

    Ok(Response::new()
        .add_attribute("action", "add_claim")
        .add_attribute("claim_id", claim.id.to_string()))
}

/// Unstake tokens. Only admin can perform this on behalf of a user.
pub fn unstake(ctx: &mut Context, msg: UnstakeMsg) -> TokenStakingResult<Response> {
    // only admin can execute this
    admin_caller_only(ctx)?;

    let user = ctx.deps.api.addr_validate(&msg.user)?;

    let user_stake = get_user_stake(ctx.deps.storage, user.clone())?;

    if user_stake < msg.amount {
        return Err(InsufficientStake);
    }

    let unstaked_amount = msg.amount;

    decrement_user_total_staked(ctx.deps.storage, user.clone(), unstaked_amount)?;
    let new_total_staked = decrement_total_staked(ctx, unstaked_amount)?;

    let release_at = calculate_release_at(ctx)?;

    let claim = add_claim(ctx.deps.storage, user, unstaked_amount, release_at)?;

    Ok(Response::new()
        .add_attribute("action", "unstake")
        .add_attribute("total_staked", new_total_staked.to_string())
        .add_attribute("claim_id", claim.id.to_string()))
}

// TODO: move to common?
fn calculate_release_at(ctx: &mut Context) -> TokenStakingResult<ReleaseAt> {
    let config = CONFIG.load(ctx.deps.storage)?;

    let release_at = match config.unlocking_period {
        Height(height) => ReleaseAt::Height((ctx.env.block.height + height).into()),
        Time(time) => ReleaseAt::Timestamp(ctx.env.block.time.plus_seconds(time)),
    };
    Ok(release_at)
}

/// Update the config. Only the current admin can execute this.
pub fn update_config(ctx: &mut Context, msg: UpdateConfigMsg) -> TokenStakingResult<Response> {
    // only admin can execute this
    admin_caller_only(ctx)?;

    let mut config = CONFIG.load(ctx.deps.storage)?;

    if let Some(new_admin) = msg.new_admin {
        config.admin = ctx.deps.api.addr_validate(&new_admin)?;
    }

    if let Some(new_token_contract) = msg.new_token_contract {
        config.token_contract = ctx.deps.api.addr_validate(&new_token_contract)?;
    }

    if let Some(new_unlocking_period) = msg.new_unlocking_period {
        config.unlocking_period = new_unlocking_period;
    }

    CONFIG.save(ctx.deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

/// Claim any unstaked tokens that are ready to be released.
pub fn claim(ctx: &mut Context, msg: ClaimMsg) -> TokenStakingResult<Response> {
    let user = ctx.deps.api.addr_validate(&msg.user)?;

    let releasable_claims =
        get_releasable_claims(ctx.deps.storage, &ctx.env.block, user.clone())?.claims;

    let token_contract = CONFIG.load(ctx.deps.storage)?.token_contract;

    let mut claim_amount = Uint128::zero();

    for claim in releasable_claims {
        claim_amount += claim.amount;

        TOKEN_CLAIMS().remove(ctx.deps.storage, claim.id.u64())?;
    }

    let send_tokens_submsg = SubMsg::new(wasm_execute(
        token_contract.to_string(),
        &cw20::Cw20ExecuteMsg::Transfer {
            recipient: user.to_string(),
            amount: claim_amount,
        },
        vec![],
    )?);

    Ok(Response::new()
        .add_attribute("action", "claim")
        .add_submessage(send_tokens_submsg))
}
