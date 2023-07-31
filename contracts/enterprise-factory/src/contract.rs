use crate::denom_membership::instantiate_denom_staking_membership_contract;
use crate::migration::migrate_config;
use crate::multisig_membership::{
    import_cw3_membership, instantiate_multisig_membership_contract,
    instantiate_new_multisig_membership,
};
use crate::nft_membership::{
    import_cw721_membership, instantiate_new_cw721_membership,
    instantiate_nft_staking_membership_contract,
};
use crate::state::{
    DaoBeingCreated, CONFIG, DAO_ADDRESSES, DAO_BEING_CREATED, DAO_ID_COUNTER, ENTERPRISE_CODE_IDS,
};
use crate::token_membership::{
    import_cw20_membership, instantiate_new_cw20_membership,
    instantiate_token_staking_membership_contract,
};
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::WasmMsg::Instantiate;
use cosmwasm_std::{
    entry_point, to_binary, wasm_execute, wasm_instantiate, Addr, Binary, Deps, DepsMut, Env,
    MessageInfo, Reply, Response, StdError, StdResult, SubMsg, Uint128, Uint64, WasmMsg,
};
use cw2::set_contract_version;
use cw_asset::AssetInfo;
use cw_storage_plus::Bound;
use cw_utils::parse_reply_instantiate_data;
use enterprise_factory_api::api::{
    AllDaosResponse, ConfigResponse, CreateDaoMembershipMsg, CreateDaoMsg, EnterpriseCodeIdsMsg,
    EnterpriseCodeIdsResponse, IsEnterpriseCodeIdMsg, IsEnterpriseCodeIdResponse, QueryAllDaosMsg,
};
use enterprise_factory_api::msg::ExecuteMsg::FinalizeDaoCreation;
use enterprise_factory_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use enterprise_factory_api::response::{
    execute_create_dao_response, execute_finalize_dao_creation_response, instantiate_response,
};
use enterprise_protocol::api::FinalizeInstantiationMsg;
use enterprise_protocol::error::DaoError::Unauthorized;
use enterprise_protocol::error::{DaoError, DaoResult};
use enterprise_protocol::msg::ExecuteMsg::FinalizeInstantiation;
use enterprise_versioning_api::api::VersionResponse;
use enterprise_versioning_api::msg::QueryMsg::LatestVersion;
use funds_distributor_api::api::UserWeight;
use itertools::Itertools;
use membership_common_api::api::WeightChangeHookMsg;
use CreateDaoMembershipMsg::{
    ImportCw20, ImportCw3, ImportCw721, NewCw20, NewCw721, NewDenom, NewMultisig,
};
use ExecuteMsg::CreateDao;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_QUERY_LIMIT: u32 = 100;
const DEFAULT_QUERY_LIMIT: u32 = 50;

pub const ENTERPRISE_INSTANTIATE_REPLY_ID: u64 = 1;
pub const CW20_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 2;
pub const CW721_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 3;
pub const MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 4;
pub const COUNCIL_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 5;
pub const FUNDS_DISTRIBUTOR_INSTANTIATE_REPLY_ID: u64 = 6;
pub const ENTERPRISE_GOVERNANCE_INSTANTIATE_REPLY_ID: u64 = 7;
pub const ENTERPRISE_GOVERNANCE_CONTROLLER_INSTANTIATE_REPLY_ID: u64 = 8;
pub const ENTERPRISE_TREASURY_INSTANTIATE_REPLY_ID: u64 = 9;
pub const ATTESTATION_INSTANTIATE_REPLY_ID: u64 = 10;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> DaoResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &msg.config)?;
    DAO_ID_COUNTER.save(deps.storage, &1u64)?;

    Ok(instantiate_response())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> DaoResult<Response> {
    match msg {
        CreateDao(msg) => create_dao(deps, env, *msg),
        FinalizeDaoCreation {} => finalize_dao_creation(deps, env, info),
    }
}

fn create_dao(deps: DepsMut, env: Env, msg: CreateDaoMsg) -> DaoResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    let latest_version_response: VersionResponse = deps
        .querier
        .query_wasm_smart(config.enterprise_versioning.to_string(), &LatestVersion {})?;

    let latest_version = latest_version_response.version;

    let enterprise_code_id = latest_version.enterprise_code_id;

    DAO_BEING_CREATED.save(
        deps.storage,
        &DaoBeingCreated {
            create_dao_msg: Some(msg.clone()),
            version_info: Some(latest_version),
            dao_asset: None,
            dao_nft: None,
            enterprise_address: None,
            initial_weights: None,
            dao_type: None,
            unlocking_period: None,
            membership_address: None,
            council_membership_address: None,
            funds_distributor_address: None,
            enterprise_governance_address: None,
            enterprise_governance_controller_address: None,
            enterprise_treasury_address: None,
            attestation_addr: None,
        },
    )?;

    let instantiate_enterprise_msg = enterprise_protocol::msg::InstantiateMsg {
        dao_metadata: msg.dao_metadata.clone(),
        enterprise_factory_contract: env.contract.address.to_string(),
        enterprise_versioning_contract: config.enterprise_versioning.to_string(),
    };
    let create_dao_submsg = SubMsg::reply_on_success(
        Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: enterprise_code_id,
            msg: to_binary(&instantiate_enterprise_msg)?,
            funds: vec![],
            label: msg.dao_metadata.name,
        },
        ENTERPRISE_INSTANTIATE_REPLY_ID,
    );

    Ok(execute_create_dao_response().add_submessage(create_dao_submsg))
}

fn finalize_dao_creation(deps: DepsMut, env: Env, info: MessageInfo) -> DaoResult<Response> {
    if info.sender != env.contract.address {
        return Err(Unauthorized);
    }

    let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;

    DAO_BEING_CREATED.save(
        deps.storage,
        &DaoBeingCreated {
            create_dao_msg: None,
            version_info: None,
            dao_asset: None,
            dao_nft: None,
            enterprise_address: None,
            initial_weights: None,
            dao_type: None,
            unlocking_period: None,
            membership_address: None,
            council_membership_address: None,
            funds_distributor_address: None,
            enterprise_governance_address: None,
            enterprise_governance_controller_address: None,
            enterprise_treasury_address: None,
            attestation_addr: None,
        },
    )?;

    let finalize_creation_submsg = SubMsg::new(wasm_execute(
        dao_being_created.require_enterprise_address()?.to_string(),
        &FinalizeInstantiation(FinalizeInstantiationMsg {
            enterprise_treasury_contract: dao_being_created
                .require_enterprise_treasury_address()?
                .to_string(),
            enterprise_governance_contract: dao_being_created
                .require_enterprise_governance_address()?
                .to_string(),
            enterprise_governance_controller_contract: dao_being_created
                .require_enterprise_governance_controller_address()?
                .to_string(),
            funds_distributor_contract: dao_being_created
                .require_funds_distributor_address()?
                .to_string(),
            membership_contract: dao_being_created.require_membership_address()?.to_string(),
            council_membership_contract: dao_being_created
                .require_council_membership_address()?
                .to_string(),
            attestation_contract: dao_being_created
                .attestation_addr
                .as_ref()
                .map(|addr| addr.to_string()),
            dao_type: dao_being_created.require_dao_type()?,
        }),
        vec![],
    )?);

    Ok(execute_finalize_dao_creation_response(
        dao_being_created.require_enterprise_address()?.to_string(),
        dao_being_created
            .require_enterprise_treasury_address()?
            .to_string(),
    )
    .add_submessage(finalize_creation_submsg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(mut deps: DepsMut, env: Env, msg: Reply) -> DaoResult<Response> {
    match msg.id {
        ENTERPRISE_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let enterprise_contract = deps.api.addr_validate(&contract_address)?;

            let id = DAO_ID_COUNTER.load(deps.storage)?;
            let next_id = id + 1;
            DAO_ID_COUNTER.save(deps.storage, &next_id)?;

            DAO_ADDRESSES.save(deps.storage, id, &enterprise_contract)?;

            // Update the admin of the DAO contract to be the DAO itself
            let update_admin_msg = SubMsg::new(WasmMsg::UpdateAdmin {
                contract_addr: enterprise_contract.to_string(),
                admin: enterprise_contract.to_string(),
            });

            let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;

            let create_dao_msg = dao_being_created.require_create_dao_msg()?;
            let version_info = dao_being_created.require_version_info()?;

            DAO_BEING_CREATED.save(
                deps.storage,
                &DaoBeingCreated {
                    enterprise_address: Some(enterprise_contract.clone()),
                    ..dao_being_created
                },
            )?;

            let enterprise_governance_controller_submsg = SubMsg::reply_on_success(
                Wasm(Instantiate {
                    admin: Some(enterprise_contract.to_string()),
                    code_id: version_info.enterprise_governance_controller_code_id,
                    msg: to_binary(&enterprise_governance_controller_api::msg::InstantiateMsg {
                        enterprise_contract: enterprise_contract.to_string(),
                        gov_config: create_dao_msg.gov_config,
                        council_gov_config: create_dao_msg.dao_council,
                        proposal_infos: None, // no proposal infos to migrate, it's a fresh DAO
                    })?,
                    funds: vec![],
                    label: "Enterprise governance controller".to_string(),
                }),
                ENTERPRISE_GOVERNANCE_CONTROLLER_INSTANTIATE_REPLY_ID,
            );

            let finalize_submsg = SubMsg::new(wasm_execute(
                env.contract.address.to_string(),
                &FinalizeDaoCreation {},
                vec![],
            )?);

            Ok(Response::new()
                .add_submessage(update_admin_msg)
                .add_submessage(enterprise_governance_controller_submsg)
                .add_submessage(finalize_submsg)
                .add_attribute("action", "instantiate_dao")
                .add_attribute("dao_address", enterprise_contract.to_string()))
        }
        CW20_CONTRACT_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let cw20_address = deps.api.addr_validate(&contract_address)?;

            let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;
            let unlocking_period = dao_being_created.require_unlocking_period()?;
            let governance_controller =
                dao_being_created.require_enterprise_governance_controller_address()?;

            DAO_BEING_CREATED.save(
                deps.storage,
                &DaoBeingCreated {
                    dao_asset: Some(AssetInfo::cw20(cw20_address.clone())),
                    ..dao_being_created
                },
            )?;

            Ok(
                Response::new().add_submessage(instantiate_token_staking_membership_contract(
                    deps,
                    cw20_address,
                    unlocking_period,
                    governance_controller.to_string(),
                )?),
            )
        }
        CW721_CONTRACT_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let cw721_address = deps.api.addr_validate(&contract_address)?;

            let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;
            let unlocking_period = dao_being_created.require_unlocking_period()?;
            let governance_controller =
                dao_being_created.require_enterprise_governance_controller_address()?;

            DAO_BEING_CREATED.save(
                deps.storage,
                &DaoBeingCreated {
                    dao_nft: Some(cw721_address.clone()),
                    ..dao_being_created
                },
            )?;

            Ok(
                Response::new().add_submessage(instantiate_nft_staking_membership_contract(
                    deps,
                    cw721_address,
                    unlocking_period,
                    governance_controller.to_string(),
                )?),
            )
        }
        MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let membership_contract = deps.api.addr_validate(&contract_address)?;

            let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;

            let governance_controller =
                dao_being_created.require_enterprise_governance_controller_address()?;

            let add_weight_change_hook_submsg = SubMsg::new(wasm_execute(
                membership_contract.to_string(),
                &membership_common_api::msg::ExecuteMsg::AddWeightChangeHook(WeightChangeHookMsg {
                    hook_addr: governance_controller.to_string(),
                }),
                vec![],
            )?);

            DAO_BEING_CREATED.save(
                deps.storage,
                &DaoBeingCreated {
                    membership_address: Some(membership_contract),
                    ..dao_being_created
                },
            )?;

            Ok(Response::new().add_submessage(add_weight_change_hook_submsg))
        }
        COUNCIL_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let council_membership_contract = deps.api.addr_validate(&contract_address)?;

            let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;

            let governance_controller =
                dao_being_created.require_enterprise_governance_controller_address()?;

            let add_weight_change_hook_submsg = SubMsg::new(wasm_execute(
                council_membership_contract.to_string(),
                &membership_common_api::msg::ExecuteMsg::AddWeightChangeHook(WeightChangeHookMsg {
                    hook_addr: governance_controller.to_string(),
                }),
                vec![],
            )?);

            DAO_BEING_CREATED.save(
                deps.storage,
                &DaoBeingCreated {
                    council_membership_address: Some(council_membership_contract),
                    ..dao_being_created
                },
            )?;

            Ok(Response::new().add_submessage(add_weight_change_hook_submsg))
        }
        FUNDS_DISTRIBUTOR_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let funds_distributor_contract = deps.api.addr_validate(&contract_address)?;

            DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
                Ok(DaoBeingCreated {
                    funds_distributor_address: Some(funds_distributor_contract),
                    ..info
                })
            })?;

            Ok(Response::new())
        }
        ENTERPRISE_GOVERNANCE_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let enterprise_governance_contract = deps.api.addr_validate(&contract_address)?;

            DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
                Ok(DaoBeingCreated {
                    enterprise_governance_address: Some(enterprise_governance_contract),
                    ..info
                })
            })?;

            Ok(Response::new())
        }
        ENTERPRISE_GOVERNANCE_CONTROLLER_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let enterprise_governance_controller_contract =
                deps.api.addr_validate(&contract_address)?;

            let dao_being_created = DAO_BEING_CREATED.load(deps.storage)?;

            let create_dao_msg = dao_being_created.require_create_dao_msg()?;
            let version_info = dao_being_created.require_version_info()?;

            let enterprise_contract = dao_being_created.require_enterprise_address()?;

            let initial_weights = dao_being_created
                .initial_weights
                .clone()
                .unwrap_or_default()
                .iter()
                .map(|user_weight| UserWeight {
                    user: user_weight.user.clone(),
                    weight: user_weight.weight,
                })
                .collect();

            let funds_distributor_submsg = SubMsg::reply_on_success(
                Wasm(Instantiate {
                    admin: Some(enterprise_contract.to_string()),
                    code_id: version_info.funds_distributor_code_id,
                    msg: to_binary(&funds_distributor_api::msg::InstantiateMsg {
                        admin: enterprise_governance_controller_contract.to_string(),
                        enterprise_contract: enterprise_contract.to_string(),
                        initial_weights,
                        minimum_eligible_weight: create_dao_msg.minimum_weight_for_rewards,
                    })?,
                    funds: vec![],
                    label: "Funds distributor".to_string(),
                }),
                FUNDS_DISTRIBUTOR_INSTANTIATE_REPLY_ID,
            );

            let enterprise_governance_submsg = SubMsg::reply_on_success(
                Wasm(Instantiate {
                    admin: Some(enterprise_contract.to_string()),
                    code_id: version_info.enterprise_governance_code_id,
                    msg: to_binary(&enterprise_governance_api::msg::InstantiateMsg {
                        admin: enterprise_governance_controller_contract.to_string(),
                    })?,
                    funds: vec![],
                    label: "Enterprise governance".to_string(),
                }),
                ENTERPRISE_GOVERNANCE_INSTANTIATE_REPLY_ID,
            );

            let mut asset_whitelist = create_dao_msg.asset_whitelist.unwrap_or_default();
            if let Some(asset) = dao_being_created.dao_asset.clone() {
                asset_whitelist.push(asset.into());
            }

            let mut nft_whitelist = create_dao_msg.nft_whitelist.unwrap_or_default();
            if let Some(nft) = dao_being_created.dao_nft.clone() {
                nft_whitelist.push(nft.to_string());
            }

            let enterprise_treasury_submsg = SubMsg::reply_on_success(
                Wasm(Instantiate {
                    admin: Some(enterprise_contract.to_string()),
                    code_id: version_info.enterprise_treasury_code_id,
                    msg: to_binary(&enterprise_treasury_api::msg::InstantiateMsg {
                        admin: enterprise_governance_controller_contract.to_string(),
                        enterprise_contract: enterprise_contract.to_string(),
                        asset_whitelist: Some(asset_whitelist),
                        nft_whitelist: Some(nft_whitelist),
                    })?,
                    funds: vec![],
                    label: "Enterprise treasury".to_string(),
                }),
                ENTERPRISE_TREASURY_INSTANTIATE_REPLY_ID,
            );

            let membership_submsg = match create_dao_msg.dao_membership {
                NewDenom(msg) => instantiate_denom_staking_membership_contract(
                    deps.branch(),
                    msg.denom,
                    msg.unlocking_period,
                    enterprise_governance_controller_contract.to_string(),
                )?,
                ImportCw20(msg) => import_cw20_membership(
                    deps.branch(),
                    msg,
                    enterprise_governance_controller_contract.to_string(),
                )?,
                NewCw20(msg) => instantiate_new_cw20_membership(deps.branch(), *msg)?,
                ImportCw721(msg) => import_cw721_membership(
                    deps.branch(),
                    msg,
                    enterprise_governance_controller_contract.to_string(),
                )?,
                NewCw721(msg) => instantiate_new_cw721_membership(deps.branch(), msg)?,
                ImportCw3(msg) => import_cw3_membership(
                    deps.branch(),
                    msg,
                    enterprise_governance_controller_contract.to_string(),
                )?,
                NewMultisig(msg) => instantiate_new_multisig_membership(
                    deps.branch(),
                    msg,
                    enterprise_governance_controller_contract.to_string(),
                )?,
            };

            let council_members = create_dao_msg
                .dao_council
                .map(|council| council.members)
                .unwrap_or_default()
                .into_iter()
                .map(|member| multisig_membership_api::api::UserWeight {
                    user: member,
                    weight: Uint128::one(),
                })
                .collect();
            let council_membership_submsg = instantiate_multisig_membership_contract(
                deps.branch(),
                council_members,
                enterprise_governance_controller_contract.to_string(),
                COUNCIL_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
            )?;

            DAO_BEING_CREATED.save(
                deps.storage,
                &DaoBeingCreated {
                    enterprise_governance_controller_address: Some(
                        enterprise_governance_controller_contract,
                    ),
                    ..dao_being_created
                },
            )?;

            let response = Response::new()
                .add_submessage(funds_distributor_submsg)
                .add_submessage(enterprise_governance_submsg)
                .add_submessage(enterprise_treasury_submsg)
                .add_submessage(membership_submsg)
                .add_submessage(council_membership_submsg);

            let response = match create_dao_msg.attestation_text {
                Some(attestation_text) => response.add_submessage(SubMsg::reply_on_success(
                    wasm_instantiate(
                        version_info.attestation_code_id,
                        &attestation_api::msg::InstantiateMsg { attestation_text },
                        vec![],
                        "Attestation contract".to_string(),
                    )?,
                    ATTESTATION_INSTANTIATE_REPLY_ID,
                )),
                None => response,
            };

            Ok(response)
        }
        ENTERPRISE_TREASURY_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let enterprise_treasury_contract = deps.api.addr_validate(&contract_address)?;

            DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
                Ok(DaoBeingCreated {
                    enterprise_treasury_address: Some(enterprise_treasury_contract),
                    ..info
                })
            })?;

            Ok(Response::new())
        }
        ATTESTATION_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let attestation_contract = deps.api.addr_validate(&contract_address)?;

            DAO_BEING_CREATED.update(deps.storage, |info| -> StdResult<DaoBeingCreated> {
                Ok(DaoBeingCreated {
                    attestation_addr: Some(attestation_contract),
                    ..info
                })
            })?;

            Ok(Response::new())
        }
        _ => Err(DaoError::Std(StdError::generic_err(
            "No such reply ID found",
        ))),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> DaoResult<Binary> {
    let response = match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?)?,
        QueryMsg::AllDaos(msg) => to_binary(&query_all_daos(deps, msg)?)?,
        QueryMsg::EnterpriseCodeIds(msg) => to_binary(&query_enterprise_code_ids(deps, msg)?)?,
        QueryMsg::IsEnterpriseCodeId(msg) => to_binary(&query_is_enterprise_code_id(deps, msg)?)?,
    };
    Ok(response)
}

pub fn query_config(deps: Deps) -> DaoResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse { config })
}

pub fn query_all_daos(deps: Deps, msg: QueryAllDaosMsg) -> DaoResult<AllDaosResponse> {
    let start_after = msg.start_after.map(Bound::exclusive);
    let limit = msg
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT);
    let addresses = DAO_ADDRESSES
        .range(deps.storage, start_after, None, Ascending)
        .take(limit as usize)
        .collect::<StdResult<Vec<(u64, Addr)>>>()?
        .into_iter()
        .map(|(_, addr)| addr)
        .collect_vec();

    Ok(AllDaosResponse { daos: addresses })
}

pub fn query_enterprise_code_ids(
    deps: Deps,
    msg: EnterpriseCodeIdsMsg,
) -> DaoResult<EnterpriseCodeIdsResponse> {
    let start_after = msg.start_after.map(Bound::exclusive);
    let limit = msg
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT);

    let code_ids = ENTERPRISE_CODE_IDS
        .range(deps.storage, start_after, None, Ascending)
        .take(limit as usize)
        .collect::<StdResult<Vec<(u64, ())>>>()?
        .into_iter()
        .map(|(code_id, _)| Uint64::from(code_id))
        .collect_vec();

    Ok(EnterpriseCodeIdsResponse { code_ids })
}

pub fn query_is_enterprise_code_id(
    deps: Deps,
    msg: IsEnterpriseCodeIdMsg,
) -> DaoResult<IsEnterpriseCodeIdResponse> {
    let is_enterprise_code_id = ENTERPRISE_CODE_IDS.has(deps.storage, msg.code_id.u64());
    Ok(IsEnterpriseCodeIdResponse {
        is_enterprise_code_id,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(mut deps: DepsMut, _env: Env, msg: MigrateMsg) -> DaoResult<Response> {
    migrate_config(deps.branch(), msg.enterprise_versioning_addr)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new())
}
