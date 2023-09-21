use crate::contract::{
    CW721_CONTRACT_INSTANTIATE_REPLY_ID, MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
};
use crate::state::{DaoBeingCreated, CONFIG, DAO_BEING_CREATED};
use crate::validate::validate_existing_cw721_contract;
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::WasmMsg::Instantiate;
use cosmwasm_std::{to_binary, Addr, DepsMut, StdResult, SubMsg};
use cw_utils::Duration;
use enterprise_factory_api::api::{ImportCw721MembershipMsg, NewCw721MembershipMsg};
use enterprise_protocol::error::DaoResult;
use nft_staking_api::msg::InstantiateMsg;

pub fn import_cw721_membership(
    deps: DepsMut,
    msg: ImportCw721MembershipMsg,
    weight_change_hooks: Option<Vec<String>>,
) -> DaoResult<SubMsg> {
    let cw721_address = deps.api.addr_validate(&msg.cw721_contract)?;

    validate_existing_cw721_contract(deps.as_ref(), cw721_address.as_ref())?;

    DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
        Ok(DaoBeingCreated {
            unlocking_period: Some(msg.unlocking_period),
            ..info
        })
    })?;

    instantiate_nft_staking_membership_contract(
        deps,
        cw721_address,
        msg.unlocking_period,
        weight_change_hooks,
    )
}

pub fn instantiate_new_cw721_membership(
    deps: DepsMut,
    msg: NewCw721MembershipMsg,
) -> DaoResult<SubMsg> {
    let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;

    let enterprise_address = dao_being_created.require_enterprise_address()?;

    DAO_BEING_CREATED.save(
        deps.storage,
        &DaoBeingCreated {
            unlocking_period: Some(msg.unlocking_period),
            ..dao_being_created
        },
    )?;

    let minter = msg.minter.unwrap_or(enterprise_address.to_string());
    let instantiate_msg = cw721_base::msg::InstantiateMsg {
        name: msg.nft_name.clone(),
        symbol: msg.nft_symbol,
        minter,
    };

    let cw721_code_id = CONFIG.load(deps.storage)?.cw721_code_id;

    let instantiate_dao_nft_submsg = SubMsg::reply_on_success(
        Wasm(Instantiate {
            admin: Some(enterprise_address.to_string()),
            code_id: cw721_code_id,
            msg: to_binary(&instantiate_msg)?,
            funds: vec![],
            label: msg.nft_name,
        }),
        CW721_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    Ok(instantiate_dao_nft_submsg)
}

pub fn instantiate_nft_staking_membership_contract(
    deps: DepsMut,
    cw721_address: Addr,
    unlocking_period: Duration,
    weight_change_hooks: Option<Vec<String>>,
) -> DaoResult<SubMsg> {
    let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;

    let enterprise_contract = dao_being_created.require_enterprise_address()?;
    let version_info = dao_being_created.require_version_info()?;

    let submsg = SubMsg::reply_on_success(
        Wasm(Instantiate {
            admin: Some(enterprise_contract.to_string()),
            code_id: version_info.nft_staking_membership_code_id,
            msg: to_binary(&InstantiateMsg {
                enterprise_contract: enterprise_contract.to_string(),
                nft_contract: cw721_address.to_string(),
                unlocking_period,
                weight_change_hooks,
            })?,
            funds: vec![],
            label: "Nft staking membership".to_string(),
        }),
        MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    Ok(submsg)
}
