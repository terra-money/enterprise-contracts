use crate::contract::MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID;
use crate::state::{DaoBeingCreated, DAO_BEING_CREATED};
use crate::validate::validate_existing_cw3_contract;
use cosmwasm_std::{wasm_instantiate, DepsMut, StdResult, SubMsg};
use cw3::Cw3QueryMsg::ListVoters;
use cw3::VoterListResponse;
use enterprise_factory_api::api::{ImportCw3MembershipMsg, NewMultisigMembershipMsg};
use enterprise_protocol::api::DaoType::Multisig;
use enterprise_protocol::error::DaoResult;
use multisig_membership_api::api::UserWeight;
use multisig_membership_api::msg::InstantiateMsg;

pub fn import_cw3_membership(deps: DepsMut, msg: ImportCw3MembershipMsg) -> DaoResult<SubMsg> {
    let cw3_address = deps.api.addr_validate(&msg.cw3_contract)?;

    validate_existing_cw3_contract(deps.as_ref(), cw3_address.as_ref())?;

    DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
        Ok(DaoBeingCreated {
            dao_type: Some(Multisig),
            ..info
        })
    })?;

    // TODO: gotta do an integration test for this
    let mut initial_weights: Vec<UserWeight> = vec![];

    let mut last_voter: Option<String> = None;
    while {
        let query_msg = ListVoters {
            start_after: last_voter.clone(),
            limit: None,
        };

        last_voter = None;

        let voters: VoterListResponse = deps
            .querier
            .query_wasm_smart(&msg.cw3_contract, &query_msg)?;

        for voter in voters.voters {
            last_voter = Some(voter.addr.clone());

            initial_weights.push(UserWeight {
                user: voter.addr,
                weight: voter.weight.into(),
            });
        }

        last_voter.is_some()
    } {}

    instantiate_multisig_membership_contract(deps, initial_weights)
}

pub fn instantiate_new_multisig_membership(
    deps: DepsMut,
    msg: NewMultisigMembershipMsg,
) -> DaoResult<SubMsg> {
    DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
        Ok(DaoBeingCreated {
            dao_type: Some(Multisig),
            ..info
        })
    })?;

    instantiate_multisig_membership_contract(deps, msg.multisig_members)
}

pub fn instantiate_multisig_membership_contract(
    deps: DepsMut,
    initial_weights: Vec<UserWeight>,
) -> DaoResult<SubMsg> {
    let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;

    let enterprise_address = dao_being_created.require_enterprise_address()?;
    let version_info = dao_being_created.require_version_info()?;

    DAO_BEING_CREATED.save(
        deps.storage,
        &DaoBeingCreated {
            initial_weights: Some(initial_weights.clone()),
            ..dao_being_created
        },
    )?;

    let submsg = SubMsg::reply_on_success(
        wasm_instantiate(
            version_info.multisig_membership_code_id,
            &InstantiateMsg {
                admin: enterprise_address.to_string(),
                initial_weights: Some(initial_weights),
            },
            vec![],
            "Multisig staking membership".to_string(),
        )?,
        MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    Ok(submsg)
}
