use crate::contract::{
    CW20_CONTRACT_INSTANTIATE_REPLY_ID, MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
};
use crate::state::{DaoBeingCreated, CONFIG, DAO_BEING_CREATED};
use crate::validate::validate_existing_cw20_contract;
use cosmwasm_std::{wasm_instantiate, Addr, DepsMut, StdResult, SubMsg, Uint128};
use cw20::{Cw20Coin, Logo, MinterResponse};
use cw_utils::Duration;
use enterprise_factory_api::api::{ImportCw20MembershipMsg, NewCw20MembershipMsg};
use enterprise_protocol::api::DaoType::Token;
use enterprise_protocol::error::DaoError::{ZeroInitialDaoBalance, ZeroInitialWeightMember};
use enterprise_protocol::error::DaoResult;
use token_staking_api::msg::InstantiateMsg;

pub fn import_cw20_membership(deps: DepsMut, msg: ImportCw20MembershipMsg) -> DaoResult<SubMsg> {
    let cw20_address = deps.api.addr_validate(&msg.cw20_contract)?;

    validate_existing_cw20_contract(deps.as_ref(), cw20_address.as_ref())?;

    DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
        Ok(DaoBeingCreated {
            dao_type: Some(Token),
            unlocking_period: Some(msg.unlocking_period),
            ..info
        })
    })?;

    instantiate_token_staking_membership_contract(deps, cw20_address, msg.unlocking_period)
}

pub fn instantiate_new_cw20_membership(
    deps: DepsMut,
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

    let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;

    let enterprise_address = dao_being_created.require_enterprise_address()?;

    DAO_BEING_CREATED.save(
        deps.storage,
        &DaoBeingCreated {
            dao_type: Some(Token),
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
                address: enterprise_address.to_string(),
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
        wasm_instantiate(cw20_code_id, &create_token_msg, vec![], msg.token_name)?,
        CW20_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    Ok(instantiate_dao_token_submsg)
}

pub fn instantiate_token_staking_membership_contract(
    deps: DepsMut,
    cw20_address: Addr,
    unlocking_period: Duration,
) -> DaoResult<SubMsg> {
    let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;
    let enterprise_address = dao_being_created.require_enterprise_address()?;
    let version_info = dao_being_created.require_version_info()?;

    let submsg = SubMsg::reply_on_success(
        wasm_instantiate(
            version_info.token_staking_membership_code_id,
            &InstantiateMsg {
                admin: enterprise_address.to_string(),
                token_contract: cw20_address.to_string(),
                unlocking_period,
            },
            vec![],
            "Token staking membership".to_string(),
        )?,
        MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    Ok(submsg)
}
