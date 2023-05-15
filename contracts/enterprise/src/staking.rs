use crate::state::{DAO_TYPE, STAKING_CONTRACT};
use cosmwasm_std::{Addr, Deps, StdError, StdResult, Timestamp, Uint128};
use cw_utils::Expiration;
use cw_utils::Expiration::{AtHeight, Never};
use enterprise_protocol::api::DaoType;
use token_staking_api::api::{UserTokenStakeParams, UserTokenStakeResponse};
use DaoType::{Multisig, Nft, Token};
use Expiration::AtTime;

pub fn query_user_cw20_stake(deps: Deps, user: Addr) -> StdResult<Uint128> {
    let staking_contract = STAKING_CONTRACT.load(deps.storage)?;

    let response: UserTokenStakeResponse = deps.querier.query_wasm_smart(
        staking_contract.to_string(),
        &token_staking_api::msg::QueryMsg::UserStake(UserTokenStakeParams {
            user: user.to_string(),
        }),
    )?;

    Ok(response.staked_amount)
}

pub fn load_total_staked(deps: Deps) -> StdResult<Uint128> {
    load_total_staked_at_expiration(deps, Never {})
}

pub fn load_total_staked_at_height(deps: Deps, height: u64) -> StdResult<Uint128> {
    load_total_staked_at_expiration(deps, AtHeight(height))
}

pub fn load_total_staked_at_time(deps: Deps, time: Timestamp) -> StdResult<Uint128> {
    load_total_staked_at_expiration(deps, AtTime(time))
}

fn load_total_staked_at_expiration(deps: Deps, expiration: Expiration) -> StdResult<Uint128> {
    let dao_type = DAO_TYPE.load(deps.storage)?;

    match dao_type {
        Token => {
            let staking_contract = STAKING_CONTRACT.load(deps.storage)?;
            let response: token_staking_api::api::TotalStakedAmountResponse =
                deps.querier.query_wasm_smart(
                    staking_contract.to_string(),
                    &token_staking_api::msg::QueryMsg::TotalStakedAmount(
                        token_staking_api::api::TotalStakedAmountParams { expiration },
                    ),
                )?;

            Ok(response.total_staked_amount)
        }
        Nft => {
            let staking_contract = STAKING_CONTRACT.load(deps.storage)?;
            let response: nft_staking_api::api::TotalStakedAmountResponse =
                deps.querier.query_wasm_smart(
                    staking_contract.to_string(),
                    &nft_staking_api::msg::QueryMsg::TotalStakedAmount(
                        nft_staking_api::api::TotalStakedAmountParams { expiration },
                    ),
                )?;

            Ok(response.total_staked_amount)
        }
        Multisig => Err(StdError::generic_err(
            "Unsupported operation for multisig DAOs",
        )),
    }
}
