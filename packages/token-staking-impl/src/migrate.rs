use crate::claims::TOKEN_CLAIMS;
use crate::config::CONFIG;
use cosmwasm_std::{wasm_execute, DepsMut, Env, Order, StdResult, SubMsg, Uint128};
use cw20::Cw20Contract;
use cw20::Cw20ExecuteMsg::Transfer;
use membership_common::total_weight::load_total_weight;
use std::ops::Sub;
use token_staking_api::api::TokenClaim;
use token_staking_api::error::TokenStakingResult;
use token_staking_api::msg::MigrateMsg;

pub fn migrate_to_v1_1_1(
    deps: DepsMut,
    env: Env,
    msg: MigrateMsg,
) -> TokenStakingResult<Vec<SubMsg>> {
    let excess_assets_recipient = match msg.move_excess_membership_assets_to {
        Some(recipient) => deps.api.addr_validate(&recipient)?,
        None => return Ok(vec![]),
    };

    let total_weight = load_total_weight(deps.storage)?;

    let total_claims: Uint128 = TOKEN_CLAIMS()
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(u64, TokenClaim)>>>()?
        .into_iter()
        .map(|(_, claim)| claim.amount)
        .sum();

    let config = CONFIG.load(deps.storage)?;
    let token_balance =
        Cw20Contract(config.token_contract.clone()).balance(&deps.querier, env.contract.address)?;

    let total_owned_by_users = total_weight.checked_add(total_claims)?;

    if token_balance > total_owned_by_users {
        // send the excess balance to treasury
        let excess_balance = token_balance.sub(total_owned_by_users);

        let send_excess_to_treasury_msg = SubMsg::new(wasm_execute(
            config.token_contract,
            &Transfer {
                recipient: excess_assets_recipient.to_string(),
                amount: excess_balance,
            },
            vec![],
        )?);

        Ok(vec![send_excess_to_treasury_msg])
    } else {
        // no excess balance, so do nothing
        Ok(vec![])
    }
}
