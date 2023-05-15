use crate::staking::CW20_STAKES;
use crate::state::{CLAIMS, DAO_GOV_CONFIG, DAO_MEMBERSHIP_CONTRACT, DAO_TYPE};
use cosmwasm_std::{
    to_binary, wasm_execute, wasm_instantiate, Addr, DepsMut, Env, Order, Reply, Response,
    StdError, StdResult, SubMsg, Uint128,
};
use cw_utils::parse_reply_instantiate_data;
use enterprise_protocol::api::{Claim, ClaimAsset, DaoType};
use enterprise_protocol::error::DaoResult;
use token_staking_api::api::{UserClaim, UserStake};
use token_staking_api::msg::Cw20HookMsg::InitializeStakers;

pub const INSTANTIATE_TOKEN_STAKING_CONTRACT_REPLY_ID: u64 = 1001;
pub const INSTANTIATE_NFT_STAKING_CONTRACT_REPLY_ID: u64 = 1002;

pub fn migrate_staking(deps: DepsMut, env: Env) -> DaoResult<Vec<SubMsg>> {
    let dao_type = DAO_TYPE.load(deps.storage)?;

    match dao_type {
        DaoType::Token => {
            let token_addr = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

            let gov_config = DAO_GOV_CONFIG.load(deps.storage)?;

            let instantiate_msg = SubMsg::reply_on_success(
                wasm_instantiate(
                    0, // TODO: use real code ID
                    &token_staking_api::msg::InstantiateMsg {
                        admin: env.contract.address.to_string(),
                        token_contract: token_addr.to_string(),
                        unlocking_period: gov_config.unlocking_period,
                    },
                    vec![],
                    "Token staking".to_string(),
                )?,
                INSTANTIATE_TOKEN_STAKING_CONTRACT_REPLY_ID,
            );

            Ok(vec![instantiate_msg])
        }
        DaoType::Nft => {
            // TODO: implement
            Ok(vec![])
        }
        DaoType::Multisig => Ok(vec![]),
    }
}

pub fn reply_instantiate_token_staking_contract(deps: DepsMut, msg: Reply) -> DaoResult<Response> {
    if msg.id != INSTANTIATE_TOKEN_STAKING_CONTRACT_REPLY_ID {
        return Err(StdError::generic_err("invalid reply ID").into());
    }

    let token_staking_contract_address = parse_reply_instantiate_data(msg)
        .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
        .contract_address;

    // TODO: store token staking contract address

    let stakers = CW20_STAKES
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(Addr, Uint128)>>>()?;

    let mut total_stake = Uint128::zero();

    let mut staking_contract_stakers = vec![];

    for (staker, stake) in stakers {
        total_stake += stake;

        staking_contract_stakers.push(UserStake {
            user: staker.to_string(),
            staked_amount: stake,
        });

        CW20_STAKES.remove(deps.storage, staker);
    }

    let token_addr = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

    let initialize_stakers_msg = SubMsg::new(wasm_execute(
        token_addr.to_string(),
        &cw20::Cw20ExecuteMsg::Send {
            contract: token_staking_contract_address.clone(),
            amount: total_stake,
            msg: to_binary(&InitializeStakers {
                stakers: staking_contract_stakers,
            })?,
        },
        vec![],
    )?);

    let all_claims = CLAIMS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(Addr, Vec<Claim>)>>>()?;

    let mut total_claims_amount = Uint128::zero();

    let mut claims: Vec<UserClaim> = vec![];

    for (user, user_claims) in all_claims {
        CLAIMS.remove(deps.storage, &user);

        for claim in user_claims {
            if let ClaimAsset::Cw20(asset) = claim.asset {
                total_claims_amount += asset.amount;

                claims.push(UserClaim {
                    user: user.to_string(),
                    claim_amount: asset.amount,
                    release_at: claim.release_at.clone(),
                });
            }
        }
    }

    let initialize_claims_msg = SubMsg::new(wasm_execute(
        token_addr.to_string(),
        &cw20::Cw20ExecuteMsg::Send {
            contract: token_staking_contract_address,
            amount: total_claims_amount,
            msg: to_binary(&token_staking_api::msg::Cw20HookMsg::AddClaims { claims })?,
        },
        vec![],
    )?);

    Ok(Response::new()
        .add_submessage(initialize_stakers_msg)
        .add_submessage(initialize_claims_msg))
}
