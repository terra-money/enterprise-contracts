use crate::claims::TOKEN_CLAIMS;
use crate::config::CONFIG;
use cosmwasm_std::{wasm_execute, Addr, Deps, DepsMut, Env, Order, StdResult, SubMsg, Uint128};
use cw20::Cw20Contract;
use cw20::Cw20ExecuteMsg::Transfer;
use enterprise_protocol::api::ComponentContractsResponse;
use enterprise_protocol::msg::QueryMsg::ComponentContracts;
use membership_common::enterprise_contract::ENTERPRISE_CONTRACT;
use membership_common::total_weight::load_total_weight;
use std::ops::{Add, Sub};
use token_staking_api::api::TokenClaim;
use token_staking_api::error::TokenStakingResult;

pub fn migrate_to_v1_1_3(deps: DepsMut, env: Env) -> TokenStakingResult<Vec<SubMsg>> {
    // TODO: accept a flag that can short-circuit this and do nothing

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

    let total_owned_by_users = total_weight.add(total_claims);

    if token_balance > total_owned_by_users {
        // send the excess balance to treasury
        let excess_balance = token_balance.sub(total_owned_by_users);

        let treasury = query_treasury_contract(deps.as_ref())?;
        let send_excess_to_treasury_msg = SubMsg::new(wasm_execute(
            config.token_contract,
            &Transfer {
                recipient: treasury.to_string(),
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

fn query_treasury_contract(deps: Deps) -> TokenStakingResult<Addr> {
    let enterprise_contract = ENTERPRISE_CONTRACT.load(deps.storage)?;

    let component_contracts: ComponentContractsResponse = deps
        .querier
        .query_wasm_smart(enterprise_contract.to_string(), &ComponentContracts {})?;

    Ok(component_contracts.enterprise_treasury_contract)
}
