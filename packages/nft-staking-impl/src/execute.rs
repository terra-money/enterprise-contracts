use crate::config::CONFIG;
use crate::nft_staking::{increment_user_total_staked, save_nft_stake, NftStake, NFT_STAKES};
use crate::total_staked::increment_total_staked;
use common::cw::Context;
use cosmwasm_std::{from_binary, Response, StdError};
use nft_staking_api::api::ReceiveNftMsg;
use nft_staking_api::error::NftStakingError::{NftTokenAlreadyStaked, Unauthorized};
use nft_staking_api::error::NftStakingResult;
use nft_staking_api::msg::Cw721HookMsg;

/// Function to execute when receiving a ReceiveNft callback from a CW721 contract.
pub fn receive_nft(ctx: &mut Context, msg: ReceiveNftMsg) -> NftStakingResult<Response> {
    let config = CONFIG.load(ctx.deps.storage)?;

    // only designated NFT contract can invoke this
    if ctx.info.sender != config.nft_contract {
        return Err(Unauthorized);
    }

    // only admin should send the actual NFT, they'll tell us which user
    if msg.sender != config.admin {
        return Err(Unauthorized);
    }

    // only admin can execute this

    match from_binary(&msg.msg) {
        Ok(Cw721HookMsg::Stake { user }) => stake_nft(ctx, msg, user),
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

    let new_user_total_staked = increment_user_total_staked(ctx.deps.storage, user)?;
    let new_total_staked = increment_total_staked(ctx)?;

    Ok(Response::new()
        .add_attribute("action", "stake_cw721")
        .add_attribute("user_total_staked", new_user_total_staked.to_string())
        .add_attribute("total_staked", new_total_staked.to_string()))
}
