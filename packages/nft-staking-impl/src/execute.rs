use crate::claims::{add_claim, get_releasable_claims, NFT_CLAIMS};
use crate::config::CONFIG;
use crate::nft_staking::{save_nft_stake, NftStake, NFT_STAKES};
use common::cw::{Context, ReleaseAt};
use cosmwasm_std::{from_binary, wasm_execute, Response, StdError, SubMsg, Uint128};
use cw721::Cw721ExecuteMsg;
use cw_utils::Duration::{Height, Time};
use membership_common::admin::{admin_caller_only, ADMIN};
use membership_common::member_weights::{
    decrement_member_weight, get_member_weight, increment_member_weight,
};
use membership_common::total_weight::{decrement_total_weight, increment_total_weight};
use membership_common::weight_change_hooks::report_weight_change_submsgs;
use membership_common_api::api::UserWeightChange;
use nft_staking_api::api::{ClaimMsg, ReceiveNftMsg, UnstakeMsg, UpdateUnlockingPeriodMsg};
use nft_staking_api::error::NftStakingError::{
    NftTokenAlreadyStaked, NoNftTokenStaked, Unauthorized,
};
use nft_staking_api::error::NftStakingResult;
use nft_staking_api::msg::Cw721HookMsg;

/// Function to execute when receiving a ReceiveNft callback from a CW721 contract.
pub fn receive_nft(ctx: &mut Context, msg: ReceiveNftMsg) -> NftStakingResult<Response> {
    let config = CONFIG.load(ctx.deps.storage)?;

    // only designated NFT contract can invoke this
    if ctx.info.sender != config.nft_contract {
        return Err(Unauthorized);
    }

    let admin = ADMIN.load(ctx.deps.storage)?;
    // only admin should send the actual NFT, they'll tell us which user
    if msg.sender != admin {
        return Err(Unauthorized);
    }

    // only admin can execute this

    match from_binary(&msg.msg) {
        Ok(Cw721HookMsg::Stake { user }) => stake_nft(ctx, msg, user),
        Ok(Cw721HookMsg::AddClaim { user, release_at }) => {
            add_nft_claim(ctx, msg, user, release_at)
        }
        _ => Err(StdError::generic_err("msg payload not recognized").into()),
    }
}

fn stake_nft(ctx: &mut Context, msg: ReceiveNftMsg, user: String) -> NftStakingResult<Response> {
    let token_id = msg.token_id;

    let existing_stake = NFT_STAKES().may_load(ctx.deps.storage, token_id.clone())?;

    if existing_stake.is_some() {
        return Err(NftTokenAlreadyStaked { token_id });
    }

    let user = ctx.deps.api.addr_validate(&user)?;

    let nft_stake = NftStake {
        staker: user.clone(),
        token_id,
    };

    save_nft_stake(ctx.deps.storage, &nft_stake)?;

    let old_weight = get_member_weight(ctx.deps.storage, user.clone())?;
    let new_weight = increment_member_weight(ctx.deps.storage, user.clone(), Uint128::one())?;
    let new_total_staked = increment_total_weight(ctx, Uint128::one())?;

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
        .add_attribute("user_total_staked", new_weight.to_string())
        .add_attribute("total_staked", new_total_staked.to_string())
        .add_submessages(report_weight_change_submsgs))
}

fn add_nft_claim(
    ctx: &mut Context,
    msg: ReceiveNftMsg,
    user: String,
    release_at: ReleaseAt,
) -> NftStakingResult<Response> {
    let token_id = msg.token_id;

    let user = ctx.deps.api.addr_validate(&user)?;

    let claim = add_claim(ctx.deps.storage, user, vec![token_id], release_at)?;

    Ok(Response::new()
        .add_attribute("action", "add_claim")
        .add_attribute("claim_id", claim.id.to_string()))
}

/// Unstake NFTs. Only admin can perform this on behalf of a user.
pub fn unstake(ctx: &mut Context, msg: UnstakeMsg) -> NftStakingResult<Response> {
    // only admin can execute this
    admin_caller_only(ctx)?;

    let user = ctx.deps.api.addr_validate(&msg.user)?;

    let old_weight = get_member_weight(ctx.deps.storage, user.clone())?;

    for token_id in &msg.nft_ids {
        let nft_stake = NFT_STAKES().may_load(ctx.deps.storage, token_id.to_string())?;

        match nft_stake {
            None => {
                return Err(NoNftTokenStaked {
                    token_id: token_id.to_string(),
                });
            }
            Some(stake) => {
                if stake.staker != user {
                    return Err(Unauthorized);
                } else {
                    NFT_STAKES().remove(ctx.deps.storage, token_id.to_string())?;
                }
            }
        }
    }

    let unstaked_amount = Uint128::from(msg.nft_ids.len() as u128);

    let new_weight = decrement_member_weight(ctx.deps.storage, user.clone(), unstaked_amount)?;

    let new_total_staked = decrement_total_weight(ctx, unstaked_amount)?;

    let release_at = calculate_release_at(ctx)?;

    let claim = add_claim(ctx.deps.storage, user.clone(), msg.nft_ids, release_at)?;

    let report_weight_change_submsgs = report_weight_change_submsgs(
        ctx,
        vec![UserWeightChange {
            user: user.to_string(),
            old_weight,
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
fn calculate_release_at(ctx: &mut Context) -> NftStakingResult<ReleaseAt> {
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
) -> NftStakingResult<Response> {
    // only admin can execute this
    admin_caller_only(ctx)?;

    let mut config = CONFIG.load(ctx.deps.storage)?;

    if let Some(new_unlocking_period) = msg.new_unlocking_period {
        config.unlocking_period = new_unlocking_period;
    }

    CONFIG.save(ctx.deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_unlocking_period"))
}

/// Claim any unstaked items that are ready to be released.
pub fn claim(ctx: &mut Context, msg: ClaimMsg) -> NftStakingResult<Response> {
    let user = ctx.deps.api.addr_validate(&msg.user)?;

    let releasable_claims =
        get_releasable_claims(ctx.deps.storage, &ctx.env.block, user.clone())?.claims;

    let nft_contract = CONFIG.load(ctx.deps.storage)?.nft_contract;

    let send_nfts_submsgs = releasable_claims
        .iter()
        .flat_map(|claim| claim.nft_ids.clone())
        .flat_map(|token_id| {
            wasm_execute(
                nft_contract.to_string(),
                &Cw721ExecuteMsg::TransferNft {
                    recipient: user.to_string(),
                    token_id,
                },
                vec![],
            )
        })
        .map(SubMsg::new)
        .collect::<Vec<SubMsg>>();

    releasable_claims
        .into_iter()
        .try_for_each(|claim| NFT_CLAIMS().remove(ctx.deps.storage, claim.id.u64()))?;

    Ok(Response::new()
        .add_attribute("action", "claim")
        .add_submessages(send_nfts_submsgs))
}
