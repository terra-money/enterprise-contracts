use crate::state::{
    ComponentContracts, COMPONENT_CONTRACTS, DAO_CREATION_DATE, DAO_METADATA, DAO_TYPE,
    DAO_VERSION, ENTERPRISE_FACTORY_CONTRACT, ENTERPRISE_VERSIONING_CONTRACT,
    IS_INSTANTIATION_FINALIZED,
};
use crate::validate::{
    enterprise_factory_caller_only, enterprise_governance_controller_caller_only,
};
use common::commons::ModifyValue::Change;
use common::cw::{Context, QueryContext};
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::WasmMsg::Migrate;
use cosmwasm_std::{
    entry_point, to_binary, wasm_instantiate, Binary, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdError, StdResult, SubMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;
use enterprise_protocol::api::{
    ComponentContractsResponse, DaoInfoResponse, FinalizeInstantiationMsg, SetAttestationMsg,
    UpdateMetadataMsg, UpgradeDaoMsg,
};
use enterprise_protocol::error::DaoError::{
    AlreadyInitialized, InvalidMigrateMsgMap, MigratingToLowerVersion,
};
use enterprise_protocol::error::{DaoError, DaoResult};
use enterprise_protocol::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use enterprise_protocol::response::{
    execute_finalize_instantiation_response, execute_remove_attestation_response,
    execute_set_attestation_response, execute_update_metadata_response,
    execute_upgrade_dao_response, instantiate_response,
};
use enterprise_versioning_api::api::{
    Version, VersionInfo, VersionParams, VersionResponse, VersionsParams, VersionsResponse,
};
use enterprise_versioning_api::msg::QueryMsg::Versions;
use serde_json::json;
use serde_json::Value::Object;
use std::str::FromStr;

pub const INSTANTIATE_ATTESTATION_REPLY_ID: u64 = 1;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DEFAULT_QUERY_LIMIT: u8 = 50;
pub const MAX_QUERY_LIMIT: u8 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> DaoResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    DAO_CREATION_DATE.save(deps.storage, &env.block.time)?;

    let enterprise_factory_contract = deps.api.addr_validate(&msg.enterprise_factory_contract)?;
    ENTERPRISE_FACTORY_CONTRACT.save(deps.storage, &enterprise_factory_contract)?;

    let enterprise_versioning_contract = deps
        .api
        .addr_validate(&msg.enterprise_versioning_contract)?;
    ENTERPRISE_VERSIONING_CONTRACT.save(deps.storage, &enterprise_versioning_contract)?;

    DAO_METADATA.save(deps.storage, &msg.dao_metadata)?;

    DAO_VERSION.save(deps.storage, &Version::from_str(CONTRACT_VERSION)?)?;

    IS_INSTANTIATION_FINALIZED.save(deps.storage, &false)?;

    Ok(instantiate_response())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> DaoResult<Response> {
    let ctx = &mut Context { deps, env, info };

    match msg {
        ExecuteMsg::FinalizeInstantiation(msg) => finalize_instantiation(ctx, msg),
        ExecuteMsg::UpdateMetadata(msg) => update_metadata(ctx, msg),
        ExecuteMsg::UpgradeDao(msg) => upgrade_dao(ctx, msg),
        ExecuteMsg::SetAttestation(msg) => set_attestation(ctx, msg),
        ExecuteMsg::RemoveAttestation {} => remove_attestation(ctx),
    }
}

fn finalize_instantiation(ctx: &mut Context, msg: FinalizeInstantiationMsg) -> DaoResult<Response> {
    let is_instantiation_finalized = IS_INSTANTIATION_FINALIZED.load(ctx.deps.storage)?;

    if is_instantiation_finalized {
        return Err(AlreadyInitialized);
    }

    enterprise_factory_caller_only(ctx)?;

    let component_contracts = ComponentContracts {
        enterprise_governance_contract: ctx
            .deps
            .api
            .addr_validate(&msg.enterprise_governance_contract)?,
        enterprise_governance_controller_contract: ctx
            .deps
            .api
            .addr_validate(&msg.enterprise_governance_controller_contract)?,
        enterprise_treasury_contract: ctx
            .deps
            .api
            .addr_validate(&msg.enterprise_treasury_contract)?,
        funds_distributor_contract: ctx
            .deps
            .api
            .addr_validate(&msg.funds_distributor_contract)?,
        membership_contract: ctx.deps.api.addr_validate(&msg.membership_contract)?,
        council_membership_contract: ctx
            .deps
            .api
            .addr_validate(&msg.council_membership_contract)?,
        attestation_contract: msg
            .attestation_contract
            .map(|addr| ctx.deps.api.addr_validate(&addr))
            .transpose()?,
    };

    COMPONENT_CONTRACTS.save(ctx.deps.storage, &component_contracts)?;

    DAO_TYPE.save(ctx.deps.storage, &msg.dao_type)?;

    IS_INSTANTIATION_FINALIZED.save(ctx.deps.storage, &true)?;

    Ok(execute_finalize_instantiation_response(
        component_contracts
            .enterprise_governance_contract
            .to_string(),
        component_contracts
            .enterprise_governance_controller_contract
            .to_string(),
        component_contracts.enterprise_treasury_contract.to_string(),
        component_contracts.funds_distributor_contract.to_string(),
        component_contracts.membership_contract.to_string(),
    ))
}

fn update_metadata(ctx: &mut Context, msg: UpdateMetadataMsg) -> DaoResult<Response> {
    enterprise_governance_controller_caller_only(ctx)?;

    let mut metadata = DAO_METADATA.load(ctx.deps.storage)?;

    if let Change(name) = msg.name {
        metadata.name = name;
    }

    if let Change(description) = msg.description {
        metadata.description = description;
    }

    if let Change(logo) = msg.logo {
        metadata.logo = logo;
    }

    if let Change(github) = msg.github_username {
        metadata.socials.github_username = github;
    }
    if let Change(twitter) = msg.twitter_username {
        metadata.socials.twitter_username = twitter;
    }
    if let Change(discord) = msg.discord_username {
        metadata.socials.discord_username = discord;
    }
    if let Change(telegram) = msg.telegram_username {
        metadata.socials.telegram_username = telegram;
    }

    DAO_METADATA.save(ctx.deps.storage, &metadata)?;

    Ok(execute_update_metadata_response())
}

fn upgrade_dao(ctx: &mut Context, msg: UpgradeDaoMsg) -> DaoResult<Response> {
    let current_version = DAO_VERSION.load(ctx.deps.storage)?;

    if current_version >= msg.new_version {
        return Err(MigratingToLowerVersion {
            current: current_version,
            target: msg.new_version,
        });
    }

    enterprise_governance_controller_caller_only(ctx)?;

    let migrate_msg_json: serde_json::Value = serde_json::from_slice(msg.migrate_msg.as_slice())
        .map_err(|e| DaoError::Std(StdError::generic_err(e.to_string())))?;

    if let Object(migrate_msgs_map) = migrate_msg_json {
        let versions =
            get_versions_between_current_and_target(ctx, current_version, msg.new_version.clone())?;

        let mut submsgs = vec![];

        for version in versions {
            let msg = migrate_msgs_map.get(&version.version.to_string());
            let migrate_msg = match msg {
                Some(msg) => to_binary(msg)?,
                None => to_binary(&json!({}))?, // if no msg was supplied, just use an empty one
            };

            submsgs.push(SubMsg::new(Wasm(Migrate {
                contract_addr: ctx.env.contract.address.to_string(),
                new_code_id: version.enterprise_code_id,
                msg: migrate_msg,
            })));
        }

        Ok(execute_upgrade_dao_response(msg.new_version.to_string()).add_submessages(submsgs))
    } else {
        Err(InvalidMigrateMsgMap)
    }
}

fn get_versions_between_current_and_target(
    ctx: &Context,
    current_version: Version,
    target_version: Version,
) -> DaoResult<Vec<VersionInfo>> {
    let enterprise_versioning = ENTERPRISE_VERSIONING_CONTRACT.load(ctx.deps.storage)?;

    let mut versions: Vec<VersionInfo> = vec![];
    let mut last_version = Some(current_version);

    loop {
        let versions_response: VersionsResponse = ctx.deps.querier.query_wasm_smart(
            enterprise_versioning.to_string(),
            &Versions(VersionsParams {
                start_after: last_version.clone(),
                limit: None,
            }),
        )?;

        if versions_response.versions.is_empty() {
            break;
        }

        last_version = versions_response
            .versions
            .last()
            .map(|info| info.version.clone());

        for version in versions_response.versions {
            if version.version > target_version {
                break;
            }

            versions.push(version.clone());

            if version.version == target_version {
                break;
            }
        }
    }

    Ok(versions)
}

fn set_attestation(ctx: &mut Context, msg: SetAttestationMsg) -> DaoResult<Response> {
    enterprise_governance_controller_caller_only(ctx)?;

    let versioning_contract = ENTERPRISE_VERSIONING_CONTRACT.load(ctx.deps.storage)?;
    let version = DAO_VERSION.load(ctx.deps.storage)?;

    let version_response: VersionResponse = ctx.deps.querier.query_wasm_smart(
        versioning_contract.to_string(),
        &enterprise_versioning_api::msg::QueryMsg::Version(VersionParams { version }),
    )?;

    let instantiate_attestation_submsg = SubMsg::reply_on_success(
        wasm_instantiate(
            version_response.version.attestation_code_id,
            &attestation_api::msg::InstantiateMsg {
                attestation_text: msg.attestation_text,
            },
            vec![],
            "Attestation contract".to_string(),
        )?,
        INSTANTIATE_ATTESTATION_REPLY_ID,
    );

    Ok(execute_set_attestation_response().add_submessage(instantiate_attestation_submsg))
}

fn remove_attestation(ctx: &mut Context) -> DaoResult<Response> {
    enterprise_governance_controller_caller_only(ctx)?;

    COMPONENT_CONTRACTS.update(
        ctx.deps.storage,
        |components| -> StdResult<ComponentContracts> {
            Ok(ComponentContracts {
                attestation_contract: None,
                ..components
            })
        },
    )?;

    Ok(execute_remove_attestation_response())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> DaoResult<Response> {
    match msg.id {
        INSTANTIATE_ATTESTATION_REPLY_ID => {
            let attestation_addr = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;

            let attestation_addr = deps.api.addr_validate(&attestation_addr)?;

            COMPONENT_CONTRACTS.update(
                deps.storage,
                |components| -> StdResult<ComponentContracts> {
                    Ok(ComponentContracts {
                        attestation_contract: Some(attestation_addr),
                        ..components
                    })
                },
            )?;

            Ok(Response::new())
        }
        _ => Err(StdError::generic_err(format!("unknown reply ID: {}", msg.id)).into()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> DaoResult<Binary> {
    let qctx = QueryContext::from(deps, env);

    let response = match msg {
        QueryMsg::DaoInfo {} => to_binary(&query_dao_info(qctx)?)?,
        QueryMsg::ComponentContracts {} => to_binary(&query_dao_info(qctx)?)?,
    };
    Ok(response)
}

pub fn query_dao_info(qctx: QueryContext) -> DaoResult<DaoInfoResponse> {
    let creation_date = DAO_CREATION_DATE.load(qctx.deps.storage)?;
    let metadata = DAO_METADATA.load(qctx.deps.storage)?;
    let dao_type = DAO_TYPE.load(qctx.deps.storage)?;
    let dao_version = DAO_VERSION.load(qctx.deps.storage)?;

    Ok(DaoInfoResponse {
        creation_date,
        metadata,
        dao_type,
        dao_version,
    })
}

pub fn query_component_contracts(qctx: QueryContext) -> DaoResult<ComponentContractsResponse> {
    let component_contracts = COMPONENT_CONTRACTS.load(qctx.deps.storage)?;
    let enterprise_factory_contract = ENTERPRISE_FACTORY_CONTRACT.load(qctx.deps.storage)?;

    Ok(ComponentContractsResponse {
        enterprise_factory_contract,
        enterprise_governance_contract: component_contracts.enterprise_governance_contract,
        enterprise_governance_controller_contract: component_contracts
            .enterprise_governance_controller_contract,
        enterprise_treasury_contract: component_contracts.enterprise_treasury_contract,
        funds_distributor_contract: component_contracts.funds_distributor_contract,
        membership_contract: component_contracts.membership_contract,
        council_membership_contract: component_contracts.council_membership_contract,
        attestation_contract: component_contracts.attestation_contract,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> DaoResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    DAO_VERSION.save(deps.storage, &Version::from_str(CONTRACT_VERSION)?)?;

    Ok(Response::new().add_attribute("action", "migrate"))
}
