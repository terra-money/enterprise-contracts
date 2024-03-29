use crate::contract::{
    CW20_CONTRACT_INSTANTIATE_REPLY_ID, MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
};
use crate::state::{DaoBeingCreated, CONFIG, DAO_BEING_CREATED};
use crate::validate::{validate_existing_cw20_contract, validate_unlocking_period};
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::WasmMsg::Instantiate;
use cosmwasm_std::{to_json_binary, Addr, DepsMut, StdResult, SubMsg, Uint128};
use cw20::{Cw20Coin, Logo, MinterResponse, TokenInfoResponse};
use cw_utils::Duration;
use enterprise_factory_api::api::{ImportCw20MembershipMsg, NewCw20MembershipMsg};
use enterprise_protocol::error::DaoError::{
    TokenDaoWithNoBalancesOrMint, ZeroInitialDaoBalance, ZeroInitialWeightMember,
};
use enterprise_protocol::error::DaoResult;
use token_staking_api::msg::InstantiateMsg;

pub fn import_cw20_membership(
    deps: DepsMut,
    msg: ImportCw20MembershipMsg,
    weight_change_hooks: Option<Vec<String>>,
) -> DaoResult<SubMsg> {
    let cw20_address = deps.api.addr_validate(&msg.cw20_contract)?;

    validate_existing_cw20_contract(deps.as_ref(), cw20_address.as_ref())?;

    // if token being imported has no holders and no way to be minted,
    // DAO is essentially locked from the get-go
    let token_info: TokenInfoResponse = deps
        .querier
        .query_wasm_smart(cw20_address.to_string(), &cw20::Cw20QueryMsg::TokenInfo {})?;
    if token_info.total_supply.is_zero() {
        let minter: Option<MinterResponse> = deps
            .querier
            .query_wasm_smart(cw20_address.to_string(), &cw20::Cw20QueryMsg::Minter {})?;
        if minter.is_none() {
            return Err(TokenDaoWithNoBalancesOrMint);
        }
    }

    DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
        Ok(DaoBeingCreated {
            unlocking_period: Some(msg.unlocking_period),
            ..info
        })
    })?;

    instantiate_token_staking_membership_contract(
        deps,
        cw20_address,
        msg.unlocking_period,
        weight_change_hooks,
    )
}

pub fn instantiate_new_cw20_membership(
    deps: DepsMut,
    enterprise_treasury_contract: Addr,
    msg: NewCw20MembershipMsg,
) -> DaoResult<SubMsg> {
    if let Some(initial_dao_balance) = msg.initial_dao_balance {
        if initial_dao_balance == Uint128::zero() {
            return Err(ZeroInitialDaoBalance);
        }
    }

    for initial_balance in msg.initial_token_balances.iter() {
        if initial_balance.amount == Uint128::zero() {
            return Err(ZeroInitialWeightMember);
        }
    }

    // if there are no initial token holders and no minter is defined,
    // DAO is essentially locked from the get-go
    if msg.initial_token_balances.is_empty() && msg.token_mint.is_none() {
        return Err(TokenDaoWithNoBalancesOrMint);
    }

    let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;

    let enterprise_address = dao_being_created.require_enterprise_address()?;

    DAO_BEING_CREATED.save(
        deps.storage,
        &DaoBeingCreated {
            unlocking_period: Some(msg.unlocking_period),
            ..dao_being_created
        },
    )?;

    let marketing = msg
        .token_marketing
        .map(|marketing| cw20_base::msg::InstantiateMarketingInfo {
            project: marketing.project,
            description: marketing.description,
            marketing: marketing
                .marketing_owner
                .or_else(|| Some(enterprise_address.to_string())),
            logo: marketing.logo_url.map(Logo::Url),
        })
        .or_else(|| {
            Some(cw20_base::msg::InstantiateMarketingInfo {
                project: None,
                description: None,
                marketing: Some(enterprise_address.to_string()),
                logo: None,
            })
        });

    let initial_balances = match msg.initial_dao_balance {
        None => msg.initial_token_balances,
        Some(initial_dao_balance) => {
            let mut token_balances = msg.initial_token_balances;
            token_balances.push(Cw20Coin {
                address: enterprise_treasury_contract.to_string(),
                amount: initial_dao_balance,
            });
            token_balances
        }
    };

    let create_token_msg = cw20_base::msg::InstantiateMsg {
        name: msg.token_name.clone(),
        symbol: msg.token_symbol,
        decimals: msg.token_decimals,
        initial_balances,
        mint: msg.token_mint.or_else(|| {
            Some(MinterResponse {
                minter: enterprise_address.to_string(),
                cap: None,
            })
        }),
        marketing,
    };

    let cw20_code_id = CONFIG.load(deps.storage)?.cw20_code_id;

    let instantiate_dao_token_submsg = SubMsg::reply_on_success(
        Wasm(Instantiate {
            admin: Some(enterprise_address.to_string()),
            code_id: cw20_code_id,
            msg: to_json_binary(&create_token_msg)?,
            funds: vec![],
            label: msg.token_name,
        }),
        CW20_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    Ok(instantiate_dao_token_submsg)
}

pub fn instantiate_token_staking_membership_contract(
    deps: DepsMut,
    cw20_address: Addr,
    unlocking_period: Duration,
    weight_change_hooks: Option<Vec<String>>,
) -> DaoResult<SubMsg> {
    let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;
    let enterprise_contract = dao_being_created.require_enterprise_address()?;
    let version_info = dao_being_created.require_version_info()?;

    validate_unlocking_period(
        dao_being_created.require_create_dao_msg()?.gov_config,
        unlocking_period,
    )?;

    let submsg = SubMsg::reply_on_success(
        Wasm(Instantiate {
            admin: Some(enterprise_contract.to_string()),
            code_id: version_info.token_staking_membership_code_id,
            msg: to_json_binary(&InstantiateMsg {
                enterprise_contract: enterprise_contract.to_string(),
                token_contract: cw20_address.to_string(),
                unlocking_period,
                weight_change_hooks,
                total_weight_by_height_checkpoints: None,
                total_weight_by_seconds_checkpoints: None,
            })?,
            funds: vec![],
            label: "Token staking membership".to_string(),
        }),
        MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    Ok(submsg)
}
