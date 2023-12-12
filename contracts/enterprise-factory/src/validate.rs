use cosmwasm_std::{Deps, StdResult};
use cw20::TokenInfoResponse;
use cw3::Cw3QueryMsg::ListVoters;
use cw3::VoterListResponse;
use cw721::Cw721QueryMsg::NumTokens;
use cw721::NumTokensResponse;
use cw_utils::Duration;
use enterprise_governance_controller_api::api::GovConfig;
use enterprise_protocol::error::DaoError::{
    InvalidExistingMultisigContract, InvalidExistingNftContract, VoteDurationLongerThanUnstaking,
};
use enterprise_protocol::error::{DaoError, DaoResult};
use DaoError::InvalidExistingTokenContract;

pub fn validate_existing_cw20_contract(deps: Deps, contract: &str) -> DaoResult<()> {
    let query = cw20::Cw20QueryMsg::TokenInfo {};
    let result: StdResult<TokenInfoResponse> = deps.querier.query_wasm_smart(contract, &query);

    result.map_err(|_| InvalidExistingTokenContract)?;

    Ok(())
}

pub fn validate_existing_cw721_contract(deps: Deps, contract: &str) -> DaoResult<()> {
    let result: StdResult<NumTokensResponse> =
        deps.querier.query_wasm_smart(contract, &NumTokens {});

    result.map_err(|_| InvalidExistingNftContract)?;

    Ok(())
}

pub fn validate_existing_cw3_contract(deps: Deps, contract: &str) -> DaoResult<()> {
    let query = ListVoters {
        start_after: None,
        limit: Some(10u32),
    };
    let result: StdResult<VoterListResponse> = deps.querier.query_wasm_smart(contract, &query);

    result.map_err(|_| InvalidExistingMultisigContract)?;

    Ok(())
}

pub fn validate_unlocking_period(
    dao_gov_config: GovConfig,
    unlocking_period: Duration,
) -> DaoResult<()> {
    if let Duration::Time(unlocking_time) = unlocking_period {
        if unlocking_time < dao_gov_config.vote_duration {
            return Err(VoteDurationLongerThanUnstaking);
        }
    }
    Ok(())
}
