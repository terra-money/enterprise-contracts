use cosmwasm_std::{Deps, Response, StdError, SubMsg, Uint128};
use cw_asset::{Asset, AssetInfoBase};

use common::cw::Context;
use enterprise_protocol::api::{IsRestrictedUserParams, IsRestrictedUserResponse};
use enterprise_protocol::msg::QueryMsg::IsRestrictedUser;
use funds_distributor_api::api::{ClaimRewardsMsg, RewardsReceiver};
use funds_distributor_api::error::DistributorError::Unauthorized;
use funds_distributor_api::error::{DistributorError, DistributorResult};
use funds_distributor_api::response::execute_claim_rewards_response;
use ibc_helpers::ics20_helpers::{
    generate_cw20_ics20_transfer_msg, generate_ics20_stargate_msg, Coin,
    DEFAULT_TRANSFER_MSG_TYPE_URL,
};
use DistributorError::RestrictedUser;

use crate::cw20_distributions::{Cw20Distribution, CW20_DISTRIBUTIONS};
use crate::native_distributions::{NativeDistribution, NATIVE_DISTRIBUTIONS};
use crate::rewards::calculate_user_reward;
use crate::state::{CW20_GLOBAL_INDICES, ENTERPRISE_CONTRACT, NATIVE_GLOBAL_INDICES};
use crate::user_weights::EFFECTIVE_USER_WEIGHTS;

/// Attempt to claim rewards for the given parameters.
///
/// Calculates rewards currently available to the user, and marks them as claimed.
///
/// Returns a Response containing submessages that will send available rewards to the user.
pub fn claim_rewards(ctx: &mut Context, msg: ClaimRewardsMsg) -> DistributorResult<Response> {
    if is_restricted_user(ctx.deps.as_ref(), msg.user.clone())? {
        return Err(RestrictedUser);
    }

    let user = ctx.deps.api.addr_validate(&msg.user)?;

    if ctx.info.sender != user {
        return Err(Unauthorized);
    }

    let user_weight = EFFECTIVE_USER_WEIGHTS
        .may_load(ctx.deps.storage, user.clone())?
        .unwrap_or_default();

    let mut assets: Vec<Asset> = vec![];

    for denom in msg.native_denoms {
        let distribution =
            NATIVE_DISTRIBUTIONS().may_load(ctx.deps.storage, (user.clone(), denom.clone()))?;
        let global_index = NATIVE_GLOBAL_INDICES
            .may_load(ctx.deps.storage, denom.clone())?
            .unwrap_or_default();

        // if no rewards for the given asset, just skip
        if global_index.is_zero() {
            continue;
        }

        let reward = calculate_user_reward(global_index, distribution, user_weight)?;

        // if no user rewards due for the given asset, just skip - no need to send or store anything
        if reward.is_zero() {
            continue;
        }

        let asset = Asset::native(denom.clone(), reward);
        assets.push(asset);

        NATIVE_DISTRIBUTIONS().save(
            ctx.deps.storage,
            (user.clone(), denom.clone()),
            &NativeDistribution {
                user: user.clone(),
                denom,
                user_index: global_index,
                pending_rewards: Uint128::zero(),
            },
        )?;
    }

    for cw20_asset in msg.cw20_assets {
        let cw20_asset = ctx.deps.api.addr_validate(&cw20_asset)?;

        let distribution =
            CW20_DISTRIBUTIONS().may_load(ctx.deps.storage, (user.clone(), cw20_asset.clone()))?;
        let global_index = CW20_GLOBAL_INDICES
            .may_load(ctx.deps.storage, cw20_asset.clone())?
            .unwrap_or_default();

        // if no rewards for the given asset, just skip
        if global_index.is_zero() {
            continue;
        }

        let reward = calculate_user_reward(global_index, distribution, user_weight)?;

        // if no user rewards due for the given asset, just skip - no need to send or store anything
        if reward.is_zero() {
            continue;
        }

        let asset = Asset::cw20(cw20_asset.clone(), reward);
        assets.push(asset);

        CW20_DISTRIBUTIONS().save(
            ctx.deps.storage,
            (user.clone(), cw20_asset.clone()),
            &Cw20Distribution {
                user: user.clone(),
                cw20_asset,
                user_index: global_index,
                pending_rewards: Uint128::zero(),
            },
        )?;
    }

    let mut submsgs: Vec<SubMsg> = vec![];

    match msg.receiver.unwrap_or_else(|| RewardsReceiver::Local {
        address: user.to_string(),
    }) {
        RewardsReceiver::Local { address } => {
            for asset in assets {
                submsgs.push(SubMsg::new(asset.transfer_msg(address.clone())?));
            }
        }
        RewardsReceiver::CrossChain(receiver) => {
            // TODO: store rewards as pending delivery or sth

            for asset in assets {
                match asset.info {
                    AssetInfoBase::Native(denom) => {
                        let send_denom_msg = generate_ics20_stargate_msg(
                            DEFAULT_TRANSFER_MSG_TYPE_URL.to_string(),
                            receiver.source_port.clone(),
                            receiver.source_channel.clone(),
                            Some(Coin {
                                denom,
                                amount: asset.amount.to_string(),
                            }),
                            ctx.env.contract.address.to_string(),
                            receiver.receiver_address.clone(),
                            ctx.env
                                .block
                                .time
                                .plus_seconds(receiver.timeout_seconds)
                                .nanos(),
                            String::new(),
                        );
                        submsgs.push(SubMsg::new(send_denom_msg))
                    }
                    AssetInfoBase::Cw20(cw20_contract) => {
                        let transfer_cw20_msg = generate_cw20_ics20_transfer_msg(
                            cw20_contract.to_string(),
                            asset.amount,
                            receiver.cw20_ics20_contract.clone(),
                            receiver.source_channel.clone(),
                            receiver.receiver_address.clone(),
                            receiver.timeout_seconds,
                        )?;
                        submsgs.push(SubMsg::new(transfer_cw20_msg));
                    }
                    _ => return Err(StdError::generic_err("Unsupported reward asset type").into()),
                }
            }
        }
    }

    Ok(execute_claim_rewards_response(user.to_string()).add_submessages(submsgs))
}

fn is_restricted_user(deps: Deps, user: String) -> DistributorResult<bool> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(deps.storage)?;

    let is_restricted_user: IsRestrictedUserResponse = deps.querier.query_wasm_smart(
        enterprise_contract.to_string(),
        &IsRestrictedUser(IsRestrictedUserParams { user }),
    )?;

    Ok(is_restricted_user.is_restricted)
}
