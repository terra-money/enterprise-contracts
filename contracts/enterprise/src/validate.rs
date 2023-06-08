use common::cw::Context;
use cosmwasm_std::StdResult;
use cw20::TokenInfoResponse;
use cw721::NumTokensResponse;
use enterprise_protocol::api::DaoType;
use enterprise_protocol::api::DaoType::{Multisig, Nft, Token};
use enterprise_protocol::error::{DaoError, DaoResult};
use DaoError::{
    InvalidExistingMultisigContract, InvalidExistingNftContract, InvalidExistingTokenContract,
};

pub fn validate_existing_dao_contract(
    ctx: &Context,
    dao_type: &DaoType,
    contract: &str,
) -> DaoResult<()> {
    match dao_type {
        Token => {
            let query = cw20::Cw20QueryMsg::TokenInfo {};
            let result: StdResult<TokenInfoResponse> =
                ctx.deps.querier.query_wasm_smart(contract, &query);

            result.map_err(|_| InvalidExistingTokenContract)?;
        }
        Nft => {
            let query = cw721::Cw721QueryMsg::NumTokens {};
            let result: StdResult<NumTokensResponse> =
                ctx.deps.querier.query_wasm_smart(contract, &query);

            result.map_err(|_| InvalidExistingNftContract)?;
        }
        Multisig => {
            let query = cw3::Cw3QueryMsg::ListVoters {
                start_after: None,
                limit: Some(10u32),
            };
            let result: StdResult<cw3::VoterListResponse> =
                ctx.deps.querier.query_wasm_smart(contract, &query);

            result.map_err(|_| InvalidExistingMultisigContract)?;
        }
    }

    Ok(())
}
