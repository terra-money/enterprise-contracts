use crate::config::CONFIG;
use common::cw::Context;
use cosmwasm_std::{from_json, to_json_binary, wasm_execute, Response, SubMsg};
use cw721::Cw721ExecuteMsg::SendNft;
use ics721_callback_proxy_api::error::Ics721CallbackProxyError::Unauthorized;
use ics721_callback_proxy_api::error::Ics721CallbackProxyResult;
use ics721_callback_proxy_api::msg::Ics721HookMsg;
use ics721_types::types::Ics721ReceiveCallbackMsg;

/// Function to execute when receiving a receive callback from an ICS721 proxy contract.
pub fn ics721_receive_callback(
    ctx: &mut Context,
    msg: Ics721ReceiveCallbackMsg,
) -> Ics721CallbackProxyResult<Response> {
    let config = CONFIG.load(ctx.deps.storage)?;

    // only designated ICS721 proxy contract can invoke this
    if ctx.info.sender != config.ics721_proxy {
        return Err(Unauthorized);
    }

    match from_json(&msg.msg) {
        Ok(Ics721HookMsg::Stake {
            user,
            membership_contract,
        }) => stake_nft(ctx, msg, user, membership_contract),
        _ => Ok(Response::new().add_attribute("action", "ics721_receive_callback_unknown")),
    }
}

fn stake_nft(
    _: &mut Context,
    msg: Ics721ReceiveCallbackMsg,
    user: String,
    membership_contract: String,
) -> Ics721CallbackProxyResult<Response> {
    let mut stake_nft_msgs: Vec<SubMsg> = vec![];

    for token_id in msg.original_packet.token_ids {
        let stake_nft_msg = SubMsg::new(wasm_execute(
            msg.nft_contract.clone(),
            &SendNft {
                contract: membership_contract.clone(),
                token_id: token_id.0.clone(),
                msg: to_json_binary(&nft_staking_api::msg::Cw721HookMsg::Stake {
                    user: user.clone(),
                })?,
            },
            vec![],
        )?);

        stake_nft_msgs.push(stake_nft_msg);
    }

    Ok(Response::new()
        .add_attribute("action", "ics721_receive_callback_stake")
        .add_attribute("user", user)
        .add_attribute("membership_contract", membership_contract)
        .add_attribute("nft_contract", msg.nft_contract)
        .add_submessages(stake_nft_msgs))
}
