use crate::contract::{
    CW721_CONTRACT_INSTANTIATE_REPLY_ID, MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
};
use crate::state::{DaoBeingCreated, CONFIG, DAO_BEING_CREATED};
use crate::validate::{validate_existing_cw721_contract, validate_unlocking_period};
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::WasmMsg::Instantiate;
use cosmwasm_std::{to_json_binary, DepsMut, StdResult, SubMsg};
use cw_utils::Duration;
use enterprise_factory_api::api::{
    ImportCw721MembershipMsg, ImportIcs721MembershipMsg, NewCw721MembershipMsg,
};
use enterprise_protocol::error::DaoResult;
use nft_staking_api::api::NftContract;
use nft_staking_api::msg::InstantiateMsg;

pub fn import_cw721_membership(
    deps: DepsMut,
    msg: ImportCw721MembershipMsg,
    weight_change_hooks: Option<Vec<String>>,
) -> DaoResult<SubMsg> {
    validate_existing_cw721_contract(deps.as_ref(), &msg.cw721_contract)?;

    DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
        Ok(DaoBeingCreated {
            unlocking_period: Some(msg.unlocking_period),
            ..info
        })
    })?;

    instantiate_nft_staking_membership_contract(
        deps,
        NftContract::Cw721 {
            contract: msg.cw721_contract,
        },
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
            msg: to_json_binary(&instantiate_msg)?,
            funds: vec![],
            label: msg.nft_name,
        }),
        CW721_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    Ok(instantiate_dao_nft_submsg)
}

pub fn import_ics721_membership(
    deps: DepsMut,
    msg: ImportIcs721MembershipMsg,
    weight_change_hooks: Option<Vec<String>>,
) -> DaoResult<SubMsg> {
    let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;

    DAO_BEING_CREATED.save(
        deps.storage,
        &DaoBeingCreated {
            unlocking_period: Some(msg.unlocking_period),
            ..dao_being_created
        },
    )?;

    instantiate_nft_staking_membership_contract(
        deps,
        NftContract::Ics721 {
            contract: msg.ics721_proxy,
            class_id: msg.class_id,
        },
        msg.unlocking_period,
        weight_change_hooks,
    )
}

pub fn instantiate_nft_staking_membership_contract(
    deps: DepsMut,
    nft_contract: NftContract,
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
            code_id: version_info.nft_staking_membership_code_id,
            msg: to_json_binary(&InstantiateMsg {
                enterprise_contract: enterprise_contract.to_string(),
                nft_contract,
                unlocking_period,
                weight_change_hooks,
                total_weight_by_height_checkpoints: None,
                total_weight_by_seconds_checkpoints: None,
            })?,
            funds: vec![],
            label: "Nft staking membership".to_string(),
        }),
        MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    Ok(submsg)
}
