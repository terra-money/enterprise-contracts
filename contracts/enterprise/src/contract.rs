use crate::state::{
    ComponentContracts, COMPONENT_CONTRACTS, DAO_CREATION_DATE, DAO_METADATA, DAO_TYPE,
    DAO_VERSION, ENTERPRISE_FACTORY_CONTRACT, ENTERPRISE_VERSIONING_CONTRACT,
    IS_INSTANTIATION_FINALIZED,
};
use crate::validate::enterprise_governance_controller_caller_only;
use attestation_api::api::{HasUserSignedParams, HasUserSignedResponse};
use attestation_api::msg::QueryMsg::HasUserSigned;
use common::commons::ModifyValue;
use common::commons::ModifyValue::Change;
use common::cw::{Context, QueryContext};
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::WasmMsg::Migrate;
use cosmwasm_std::{
    entry_point, to_json_binary, wasm_instantiate, Binary, CosmosMsg, Deps, DepsMut, Empty, Env,
    MessageInfo, Reply, Response, StdError, StdResult, SubMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;
use enterprise_protocol::api::{
    ComponentContractsResponse, DaoInfoResponse, DaoType, ExecuteMsgsMsg, FinalizeInstantiationMsg,
    IsRestrictedUserParams, IsRestrictedUserResponse, SetAttestationMsg, UpdateConfigMsg,
    UpdateMetadataMsg, UpgradeDaoMsg,
};
use enterprise_protocol::error::DaoError::{
    AlreadyInitialized, DuplicateVersionMigrateMsgFound, MigratingToLowerVersion, Unauthorized,
};
use enterprise_protocol::error::DaoResult;
use enterprise_protocol::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use enterprise_protocol::response::{
    execute_execute_msgs_response, execute_finalize_instantiation_response,
    execute_remove_attestation_response, execute_set_attestation_response,
    execute_update_config_response, execute_update_metadata_response, execute_upgrade_dao_response,
    instantiate_response,
};
use enterprise_versioning_api::api::{
    Version, VersionInfo, VersionParams, VersionResponse, VersionsParams, VersionsResponse,
};
use enterprise_versioning_api::msg::QueryMsg::Versions;
use std::collections::HashMap;
use std::ops::Not;
use DaoType::Nft;
use ModifyValue::NoChange;

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

    DAO_CREATION_DATE.save(
        deps.storage,
        &msg.dao_creation_date.unwrap_or(env.block.time),
    )?;

    let enterprise_factory_contract = deps.api.addr_validate(&msg.enterprise_factory_contract)?;
    ENTERPRISE_FACTORY_CONTRACT.save(deps.storage, &enterprise_factory_contract)?;

    let enterprise_versioning_contract = deps
        .api
        .addr_validate(&msg.enterprise_versioning_contract)?;
    ENTERPRISE_VERSIONING_CONTRACT.save(deps.storage, &enterprise_versioning_contract)?;

    DAO_METADATA.save(deps.storage, &msg.dao_metadata)?;

    DAO_VERSION.save(deps.storage, &msg.dao_version)?;

    DAO_TYPE.save(deps.storage, &msg.dao_type)?;

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
        ExecuteMsg::UpdateConfig(msg) => update_config(ctx, msg),
        ExecuteMsg::SetAttestation(msg) => set_attestation(ctx, msg),
        ExecuteMsg::RemoveAttestation {} => remove_attestation(ctx),
        ExecuteMsg::ExecuteMsgs(msg) => execute_msgs(ctx, msg),
    }
}

fn finalize_instantiation(ctx: &mut Context, msg: FinalizeInstantiationMsg) -> DaoResult<Response> {
    let is_instantiation_finalized = IS_INSTANTIATION_FINALIZED.load(ctx.deps.storage)?;

    if is_instantiation_finalized {
        return Err(AlreadyInitialized);
    }

    let contract_info = ctx
        .deps
        .querier
        .query_wasm_contract_info(ctx.env.contract.address.to_string())?;

    if ctx.deps.api.addr_validate(&contract_info.creator)? != ctx.info.sender {
        return Err(Unauthorized);
    }

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
        enterprise_outposts_contract: ctx
            .deps
            .api
            .addr_validate(&msg.enterprise_outposts_contract)?,
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

    IS_INSTANTIATION_FINALIZED.save(ctx.deps.storage, &true)?;

    Ok(execute_finalize_instantiation_response(
        component_contracts
            .attestation_contract
            .map(|it| it.to_string()),
        component_contracts
            .enterprise_governance_contract
            .to_string(),
        component_contracts
            .enterprise_governance_controller_contract
            .to_string(),
        component_contracts.enterprise_treasury_contract.to_string(),
        component_contracts.funds_distributor_contract.to_string(),
        component_contracts.membership_contract.to_string(),
        component_contracts.council_membership_contract.to_string(),
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

    let mut migrate_msgs_map: HashMap<Version, Binary> = HashMap::new();

    for version_migrate_msg in msg.migrate_msgs {
        let existing_version_migrate_msg = migrate_msgs_map.insert(
            version_migrate_msg.version.clone(),
            version_migrate_msg.migrate_msg,
        );
        if existing_version_migrate_msg.is_some() {
            return Err(DuplicateVersionMigrateMsgFound {
                version: version_migrate_msg.version,
            });
        }
    }

    let versions =
        get_versions_between_current_and_target(ctx, current_version, msg.new_version.clone())?;

    let mut submsgs = vec![];

    for version in versions {
        let msg = migrate_msgs_map.get(&version.version);
        let migrate_msg = match msg {
            Some(msg) => Clone::clone(msg),
            None => to_json_binary(&Empty {})?, // if no msg was supplied, just use an empty one
        };

        submsgs.push(SubMsg::new(Wasm(Migrate {
            contract_addr: ctx.env.contract.address.to_string(),
            new_code_id: version.enterprise_code_id,
            msg: migrate_msg,
        })));
    }

    // TODO: what if someone wants to check a version in one of the intermediary steps of upgrading (e.g. upgrading from 1.0.0 to 3.0.0, on the 2.0.0 mid-step the version they'll get is incorrect)?
    DAO_VERSION.save(ctx.deps.storage, &msg.new_version)?;

    Ok(execute_upgrade_dao_response(msg.new_version.to_string()).add_submessages(submsgs))
}

fn get_versions_between_current_and_target(
    ctx: &Context,
    current_version: Version,
    target_version: Version,
) -> DaoResult<Vec<VersionInfo>> {
    let enterprise_versioning = ENTERPRISE_VERSIONING_CONTRACT.load(ctx.deps.storage)?;

    let mut versions: Vec<VersionInfo> = vec![];
    let mut last_version = Some(current_version);

    'outer: loop {
        let versions_response: VersionsResponse = ctx.deps.querier.query_wasm_smart(
            enterprise_versioning.to_string(),
            &Versions(VersionsParams {
                start_after: last_version.clone(),
                limit: Some(MAX_QUERY_LIMIT as u32),
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
                break 'outer;
            }

            versions.push(version.clone());

            if version.version == target_version {
                break 'outer;
            }
        }
    }

    Ok(versions)
}

fn update_config(ctx: &mut Context, msg: UpdateConfigMsg) -> DaoResult<Response> {
    enterprise_governance_controller_caller_only(ctx)?;

    let old_versioning = ENTERPRISE_VERSIONING_CONTRACT.load(ctx.deps.storage)?;

    let new_versioning_addr = match msg.new_versioning_contract {
        Change(versioning) => {
            let new_versioning = ctx.deps.api.addr_validate(&versioning)?;
            ENTERPRISE_VERSIONING_CONTRACT.save(ctx.deps.storage, &new_versioning)?;

            new_versioning.to_string()
        }
        NoChange => old_versioning.to_string(),
    };

    let old_factory = ENTERPRISE_FACTORY_CONTRACT.load(ctx.deps.storage)?;

    let new_factory_addr = match msg.new_factory_contract {
        Change(factory) => {
            let new_factory = ctx.deps.api.addr_validate(&factory)?;
            ENTERPRISE_FACTORY_CONTRACT.save(ctx.deps.storage, &new_factory)?;

            new_factory.to_string()
        }
        NoChange => old_factory.to_string(),
    };

    Ok(execute_update_config_response(
        old_versioning.to_string(),
        new_versioning_addr,
        old_factory.to_string(),
        new_factory_addr,
    ))
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

fn execute_msgs(ctx: &mut Context, msg: ExecuteMsgsMsg) -> DaoResult<Response> {
    enterprise_governance_controller_caller_only(ctx)?;

    let submsgs = msg
        .msgs
        .into_iter()
        .map(|msg| serde_json_wasm::from_str::<CosmosMsg>(&msg).map(SubMsg::new))
        .collect::<serde_json_wasm::de::Result<Vec<SubMsg>>>()?;

    Ok(execute_execute_msgs_response().add_submessages(submsgs))
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
        QueryMsg::DaoInfo {} => to_json_binary(&query_dao_info(qctx)?)?,
        QueryMsg::ComponentContracts {} => to_json_binary(&query_component_contracts(qctx)?)?,
        QueryMsg::IsRestrictedUser(params) => {
            to_json_binary(&query_is_restricted_user(qctx, params)?)?
        }
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
    let enterprise_versioning_contract = ENTERPRISE_VERSIONING_CONTRACT.load(qctx.deps.storage)?;

    Ok(ComponentContractsResponse {
        enterprise_factory_contract,
        enterprise_versioning_contract,
        enterprise_governance_contract: component_contracts.enterprise_governance_contract,
        enterprise_governance_controller_contract: component_contracts
            .enterprise_governance_controller_contract,
        enterprise_outposts_contract: component_contracts.enterprise_outposts_contract,
        enterprise_treasury_contract: component_contracts.enterprise_treasury_contract,
        funds_distributor_contract: component_contracts.funds_distributor_contract,
        membership_contract: component_contracts.membership_contract,
        council_membership_contract: component_contracts.council_membership_contract,
        attestation_contract: component_contracts.attestation_contract,
    })
}

/// Query whether a user should be restricted from certain DAO actions, such as governance and
/// rewards claiming.
/// Is determined by checking if there is an attestation, and if the user has signed it or not.
pub fn query_is_restricted_user(
    qctx: QueryContext,
    params: IsRestrictedUserParams,
) -> DaoResult<IsRestrictedUserResponse> {
    let component_contracts = COMPONENT_CONTRACTS.load(qctx.deps.storage)?;

    let is_restricted = match component_contracts.attestation_contract {
        None => false,
        Some(attestation_contract) => {
            let has_user_signed_response: HasUserSignedResponse =
                qctx.deps.querier.query_wasm_smart(
                    attestation_contract.to_string(),
                    &HasUserSigned(HasUserSignedParams { user: params.user }),
                )?;

            has_user_signed_response.has_signed.not()
        }
    };

    Ok(IsRestrictedUserResponse { is_restricted })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> DaoResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let versioning_contract = ENTERPRISE_VERSIONING_CONTRACT.load(deps.storage)?;

    // TODO: not optimal, what if someone used a different version deployment?
    let version_info: VersionResponse = deps.querier.query_wasm_smart(
        versioning_contract.to_string(),
        &enterprise_versioning_api::msg::QueryMsg::Version(VersionParams {
            version: Version {
                major: 1,
                minor: 2,
                patch: 0,
            },
        }),
    )?;

    let component_contracts = COMPONENT_CONTRACTS.load(deps.storage)?;

    let migrate_governance_controller_msg = SubMsg::new(Wasm(Migrate {
        contract_addr: component_contracts
            .enterprise_governance_controller_contract
            .to_string(),
        new_code_id: version_info
            .version
            .enterprise_governance_controller_code_id,
        msg: to_json_binary(&enterprise_governance_controller_api::msg::MigrateMsg {})?,
    }));

    let mut response = Response::new()
        .add_attribute("action", "migrate")
        .add_submessage(migrate_governance_controller_msg);

    let dao_type = DAO_TYPE.load(deps.storage)?;

    if dao_type == Nft {
        let migrate_nft_membership_msg = SubMsg::new(Wasm(Migrate {
            contract_addr: component_contracts.membership_contract.to_string(),
            new_code_id: version_info.version.nft_staking_membership_code_id,
            msg: to_json_binary(&nft_staking_api::msg::MigrateMsg {})?,
        }));
        response = response.add_submessage(migrate_nft_membership_msg);
    }

    Ok(response)
}
