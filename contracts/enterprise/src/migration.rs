use crate::contract::{
    COUNCIL_MEMBERSHIP_REPLY_ID, ENTERPRISE_GOVERNANCE_CONTROLLER_REPLY_ID,
    ENTERPRISE_TREASURY_REPLY_ID, MEMBERSHIP_REPLY_ID,
};
use crate::state::{
    ComponentContracts, COMPONENT_CONTRACTS, DAO_TYPE, ENTERPRISE_FACTORY_CONTRACT,
    ENTERPRISE_VERSIONING_CONTRACT,
};
use common::cw::Context;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::WasmMsg::{Instantiate, Migrate};
use cosmwasm_std::{
    to_binary, wasm_execute, Addr, Decimal, DepsMut, Env, Response, StdError, StdResult, SubMsg,
    Uint128, Uint64,
};
use cw_asset::{Asset, AssetInfoUnchecked};
use cw_storage_plus::{Item, Map};
use cw_utils::Duration;
use enterprise_factory_api::api::ConfigResponse;
use enterprise_governance_controller_api::api::{
    DaoCouncilSpec, GovConfig, ProposalActionType, ProposalId, ProposalInfo,
};
use enterprise_protocol::api::DaoType;
use enterprise_protocol::api::DaoType::Token;
use enterprise_protocol::error::DaoError::{Std, Unauthorized};
use enterprise_protocol::error::DaoResult;
use enterprise_protocol::msg::ExecuteMsg::FinalizeMigration;
use enterprise_treasury_api::api::UpdateConfigMsg;
use enterprise_versioning_api::api::{Version, VersionInfo, VersionParams, VersionResponse};
use membership_common_api::api::WeightChangeHookMsg;
use multisig_membership_api::api::UserWeight;

const NATIVE_ASSET_WHITELIST: Map<String, ()> = Map::new("native_asset_whitelist");
const CW20_ASSET_WHITELIST: Map<Addr, ()> = Map::new("cw20_asset_whitelist");
const CW1155_ASSET_WHITELIST: Map<(Addr, String), ()> = Map::new("cw1155_asset_whitelist");

const NFT_WHITELIST: Map<Addr, ()> = Map::new("nft_whitelist");

const DAO_GOV_CONFIG: Item<DaoGovConfig> = Item::new("dao_gov_config");
const DAO_COUNCIL: Item<Option<DaoCouncil>> = Item::new("dao_council");

const DAO_MEMBERSHIP_CONTRACT: Item<Addr> = Item::new("dao_membership_contract");

const DAO_CODE_VERSION: Item<Uint64> = Item::new("dao_code_version");

const MULTISIG_MEMBERS: Map<Addr, Uint128> = Map::new("multisig_members");

const ENTERPRISE_GOVERNANCE_CONTRACT: Item<Addr> = Item::new("enterprise_governance_contract");
const FUNDS_DISTRIBUTOR_CONTRACT: Item<Addr> = Item::new("funds_distributor_contract");

const PROPOSAL_INFOS: Map<ProposalId, ProposalInfo> = Map::new("proposal_infos");

#[cw_serde]
struct MigrationInfo {
    pub version_info: VersionInfo,
    pub enterprise_governance_controller_contract: Option<Addr>,
    pub enterprise_treasury_contract: Option<Addr>,
    pub membership_contract: Option<Addr>,
    pub council_membership_contract: Option<Addr>,
}
const MIGRATION_INFO: Item<MigrationInfo> = Item::new("migration_info");

#[cw_serde]
pub struct DaoGovConfig {
    /// Portion of total available votes cast in a proposal to consider it valid
    /// e.g. quorum of 30% means that 30% of all available votes have to be cast in the proposal,
    /// otherwise it fails automatically when it expires
    pub quorum: Decimal,
    /// Portion of votes assigned to a single option from all the votes cast in the given proposal
    /// required to determine the 'winning' option
    /// e.g. 51% threshold means that an option has to have at least 51% of the cast votes to win
    pub threshold: Decimal,
    /// Portion of votes assigned to veto option from all the votes cast in the given proposal
    /// required to veto the proposal.
    /// If None, will default to the threshold set for all proposal options.
    pub veto_threshold: Option<Decimal>,
    /// Duration of proposals before they end, expressed in seconds
    pub vote_duration: u64,
    /// Duration that has to pass for unstaked membership tokens to be claimable
    pub unlocking_period: Duration,
    /// Optional minimum amount of DAO's governance unit to be required to create a deposit.
    pub minimum_deposit: Option<Uint128>,
    /// If set to true, this will allow DAOs to execute proposals that have reached quorum and
    /// threshold, even before their voting period ends.
    pub allow_early_proposal_execution: bool,
}

#[cw_serde]
pub struct DaoCouncil {
    pub members: Vec<Addr>,
    pub allowed_proposal_action_types: Vec<ProposalActionType>,
    pub quorum: Decimal,
    pub threshold: Decimal,
}

pub fn migrate_to_rewrite(mut deps: DepsMut, env: Env) -> DaoResult<Vec<SubMsg>> {
    let enterprise_factory = ENTERPRISE_FACTORY_CONTRACT.load(deps.storage)?;
    let enterprise_factory_config: ConfigResponse = deps.querier.query_wasm_smart(
        enterprise_factory.to_string(),
        &enterprise_factory_api::msg::QueryMsg::Config {},
    )?;

    let enterprise_versioning = enterprise_factory_config.config.enterprise_versioning;
    ENTERPRISE_VERSIONING_CONTRACT.save(deps.storage, &enterprise_versioning)?;

    let version_info: VersionResponse = deps.querier.query_wasm_smart(
        enterprise_versioning.to_string(),
        &enterprise_versioning_api::msg::QueryMsg::Version(VersionParams {
            version: Version {
                major: 1,
                minor: 0,
                patch: 0,
            },
        }),
    )?;

    MIGRATION_INFO.save(
        deps.storage,
        &MigrationInfo {
            version_info: version_info.version.clone(),
            enterprise_governance_controller_contract: None,
            enterprise_treasury_contract: None,
            membership_contract: None,
            council_membership_contract: None,
        },
    )?;

    let governance_controller_submsg =
        create_governance_controller_contract(deps.branch(), env.clone())?;

    let treasury_submsg = create_treasury_contract(
        deps.branch(),
        env.clone(),
        version_info.version.enterprise_treasury_code_id,
    )?;

    let dao_council_membership_submsg = create_dao_council_membership_contract(
        deps.branch(),
        env.clone(),
        version_info.version.multisig_membership_code_id,
    )?;

    let membership_submsg = create_enterprise_membership_contract(
        deps.branch(),
        env.clone(),
        version_info.version.token_staking_membership_code_id,
        version_info.version.nft_staking_membership_code_id,
        version_info.version.multisig_membership_code_id,
    )?;

    let finalize_migration_submsg = SubMsg::new(wasm_execute(
        env.contract.address.to_string(),
        &FinalizeMigration {},
        vec![],
    )?);

    Ok(vec![
        governance_controller_submsg,
        dao_council_membership_submsg,
        treasury_submsg,
        membership_submsg,
        finalize_migration_submsg,
    ])
}

pub fn create_treasury_contract(
    deps: DepsMut,
    env: Env,
    treasury_code_id: u64,
) -> DaoResult<SubMsg> {
    let mut asset_whitelist = NATIVE_ASSET_WHITELIST
        .range(deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<(String, ())>>>()?
        .into_iter()
        .map(|(denom, _)| AssetInfoUnchecked::native(denom))
        .collect::<Vec<AssetInfoUnchecked>>();

    let mut cw20_asset_whitelist = CW20_ASSET_WHITELIST
        .range(deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<(Addr, ())>>>()?
        .into_iter()
        .map(|(addr, _)| AssetInfoUnchecked::cw20(addr.to_string()))
        .collect::<Vec<AssetInfoUnchecked>>();

    let mut cw1155_asset_whitelist = CW1155_ASSET_WHITELIST
        .range(deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<((Addr, String), ())>>>()?
        .into_iter()
        .map(|((addr, token_id), _)| AssetInfoUnchecked::cw1155(addr.to_string(), token_id))
        .collect::<Vec<AssetInfoUnchecked>>();

    asset_whitelist.append(&mut cw20_asset_whitelist);
    asset_whitelist.append(&mut cw1155_asset_whitelist);

    let nft_whitelist = NFT_WHITELIST
        .range(deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<(Addr, ())>>>()?
        .into_iter()
        .map(|(addr, _)| addr.to_string())
        .collect();

    let submsg = SubMsg::reply_on_success(
        Wasm(Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: treasury_code_id,
            msg: to_binary(&enterprise_treasury_api::msg::InstantiateMsg {
                admin: env.contract.address.to_string(),
                enterprise_contract: env.contract.address.to_string(),
                asset_whitelist: Some(asset_whitelist),
                nft_whitelist: Some(nft_whitelist),
            })?,
            funds: vec![],
            label: "Enterprise treasury".to_string(),
        }),
        ENTERPRISE_TREASURY_REPLY_ID,
    );

    Ok(submsg)
}

pub fn treasury_contract_created(deps: DepsMut, treasury_contract: Addr) -> DaoResult<Response> {
    let migration_info = MIGRATION_INFO.load(deps.storage)?;

    MIGRATION_INFO.save(
        deps.storage,
        &MigrationInfo {
            enterprise_treasury_contract: Some(treasury_contract),
            ..migration_info
        },
    )?;

    Ok(Response::new())
}

pub fn create_dao_council_membership_contract(
    deps: DepsMut,
    env: Env,
    multisig_membership_code_id: u64,
) -> DaoResult<SubMsg> {
    let dao_council = DAO_COUNCIL.load(deps.storage)?;

    let council_members = dao_council
        .map(|council| council.members)
        .unwrap_or_default()
        .into_iter()
        .map(|member| UserWeight {
            user: member.to_string(),
            weight: Uint128::one(),
        })
        .collect();

    let instantiate_dao_council_membership = SubMsg::reply_on_success(
        Wasm(Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: multisig_membership_code_id,
            msg: to_binary(&multisig_membership_api::msg::InstantiateMsg {
                admin: env.contract.address.to_string(),
                initial_weights: Some(council_members),
            })?,
            funds: vec![],
            label: "Dao council membership".to_string(),
        }),
        COUNCIL_MEMBERSHIP_REPLY_ID,
    );

    Ok(instantiate_dao_council_membership)
}

pub fn council_membership_contract_created(
    deps: DepsMut,
    council_membership_contract: Addr,
) -> DaoResult<Response> {
    let migration_info = MIGRATION_INFO.load(deps.storage)?;
    MIGRATION_INFO.save(
        deps.storage,
        &MigrationInfo {
            council_membership_contract: Some(council_membership_contract),
            ..migration_info
        },
    )?;

    Ok(Response::new())
}

pub fn create_governance_controller_contract(deps: DepsMut, env: Env) -> DaoResult<SubMsg> {
    let version_info = MIGRATION_INFO.load(deps.storage)?.version_info;
    let gov_config = DAO_GOV_CONFIG.load(deps.storage)?;
    let dao_council = DAO_COUNCIL.load(deps.storage)?;

    let proposal_infos = PROPOSAL_INFOS
        .range(deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<(ProposalId, ProposalInfo)>>>()?;

    let submsg = SubMsg::reply_on_success(
        Wasm(Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: version_info.enterprise_governance_controller_code_id,
            msg: to_binary(&enterprise_governance_controller_api::msg::InstantiateMsg {
                enterprise_contract: env.contract.address.to_string(),
                gov_config: GovConfig {
                    quorum: gov_config.quorum,
                    threshold: gov_config.threshold,
                    veto_threshold: gov_config.veto_threshold,
                    vote_duration: gov_config.vote_duration,
                    unlocking_period: gov_config.unlocking_period,
                    minimum_deposit: gov_config.minimum_deposit,
                    allow_early_proposal_execution: gov_config.allow_early_proposal_execution,
                },
                council_gov_config: dao_council.map(|council| DaoCouncilSpec {
                    members: vec![], // TODO: this works, but we really shouldn't have this field when it's meaningless
                    allowed_proposal_action_types: Some(council.allowed_proposal_action_types),
                    quorum: council.quorum,
                    threshold: council.threshold,
                }),
                proposal_infos: Some(proposal_infos),
            })?,
            funds: vec![],
            label: "Enterprise governance controller".to_string(),
        }),
        ENTERPRISE_GOVERNANCE_CONTROLLER_REPLY_ID,
    );

    Ok(submsg)
}

pub fn governance_controller_contract_created(
    deps: DepsMut,
    governance_controller_contract: Addr,
) -> DaoResult<Response> {
    let migration_info = MIGRATION_INFO.load(deps.storage)?;

    MIGRATION_INFO.save(
        deps.storage,
        &MigrationInfo {
            enterprise_governance_controller_contract: Some(governance_controller_contract.clone()),
            ..migration_info
        },
    )?;

    let dao_type = DAO_TYPE.load(deps.storage)?;
    let response = if dao_type == Token {
        let deposit_amount: Uint128 = PROPOSAL_INFOS
            .range(deps.storage, None, None, Ascending)
            .collect::<StdResult<Vec<(ProposalId, ProposalInfo)>>>()?
            .into_iter()
            .filter_map(|(_, proposal_info)| proposal_info.proposal_deposit)
            .map(|deposit| deposit.amount)
            .sum();

        let dao_token = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

        let send_deposits_submsg = Asset::cw20(dao_token, deposit_amount)
            .transfer_msg(governance_controller_contract.to_string())?;

        Response::new().add_submessage(SubMsg::new(send_deposits_submsg))
    } else {
        Response::new()
    };

    Ok(response)
}

pub fn create_enterprise_membership_contract(
    deps: DepsMut,
    env: Env,
    token_membership_code_id: u64,
    nft_membership_code_id: u64,
    multisig_membership_code_id: u64,
) -> DaoResult<SubMsg> {
    let gov_config = DAO_GOV_CONFIG.load(deps.storage)?;

    let dao_type = DAO_TYPE.load(deps.storage)?;

    let msg = match dao_type {
        DaoType::Token => {
            let cw20_contract = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

            Wasm(Instantiate {
                admin: Some(env.contract.address.to_string()),
                code_id: token_membership_code_id,
                msg: to_binary(&token_staking_api::msg::InstantiateMsg {
                    admin: env.contract.address.to_string(),
                    token_contract: cw20_contract.to_string(),
                    unlocking_period: gov_config.unlocking_period,
                })?,
                funds: vec![],
                label: "Token staking membership".to_string(),
            })
        }
        DaoType::Nft => {
            let cw721_contract = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

            Wasm(Instantiate {
                admin: Some(env.contract.address.to_string()),
                code_id: nft_membership_code_id,
                msg: to_binary(&nft_staking_api::msg::InstantiateMsg {
                    admin: env.contract.address.to_string(),
                    nft_contract: cw721_contract.to_string(),
                    unlocking_period: gov_config.unlocking_period,
                })?,
                funds: vec![],
                label: "NFT staking membership".to_string(),
            })
        }
        DaoType::Multisig => {
            let initial_weights = MULTISIG_MEMBERS
                .range(deps.storage, None, None, Ascending)
                .collect::<StdResult<Vec<(Addr, Uint128)>>>()?
                .into_iter()
                .map(|(user, weight)| UserWeight {
                    user: user.to_string(),
                    weight,
                })
                .collect();

            Wasm(Instantiate {
                admin: Some(env.contract.address.to_string()),
                code_id: multisig_membership_code_id,
                msg: to_binary(&multisig_membership_api::msg::InstantiateMsg {
                    admin: env.contract.address.to_string(),
                    initial_weights: Some(initial_weights),
                })?,
                funds: vec![],
                label: "Multisig membership".to_string(),
            })
        }
    };

    let submsg = SubMsg::reply_on_success(msg, MEMBERSHIP_REPLY_ID);

    Ok(submsg)
}

pub fn membership_contract_created(
    deps: DepsMut,
    membership_contract: Addr,
) -> DaoResult<Response> {
    let migration_info = MIGRATION_INFO.load(deps.storage)?;

    MIGRATION_INFO.save(
        deps.storage,
        &MigrationInfo {
            membership_contract: Some(membership_contract),
            ..migration_info
        },
    )?;

    Ok(Response::new())
}

pub fn finalize_migration(ctx: &mut Context) -> DaoResult<Response> {
    if ctx.info.sender != ctx.env.contract.address {
        return Err(Unauthorized);
    }

    let migration_info = MIGRATION_INFO.load(ctx.deps.storage)?;

    let version_info = migration_info.version_info;

    let governance_controller_contract = migration_info
        .enterprise_governance_controller_contract
        .ok_or(Std(StdError::generic_err(
        "missing governance controller address",
    )))?;
    let treasury_contract = migration_info
        .enterprise_treasury_contract
        .ok_or(Std(StdError::generic_err("missing treasury address")))?;
    let membership_contract = migration_info
        .membership_contract
        .ok_or(Std(StdError::generic_err("missing membership address")))?;
    let council_membership_contract =
        migration_info
            .council_membership_contract
            .ok_or(Std(StdError::generic_err(
                "missing council membership address",
            )))?;

    let enterprise_governance_contract = ENTERPRISE_GOVERNANCE_CONTRACT.load(ctx.deps.storage)?;
    let funds_distributor_contract = FUNDS_DISTRIBUTOR_CONTRACT.load(ctx.deps.storage)?;

    let update_treasury_admin_submsg = SubMsg::new(wasm_execute(
        treasury_contract.to_string(),
        &enterprise_treasury_api::msg::ExecuteMsg::UpdateConfig(UpdateConfigMsg {
            new_admin: governance_controller_contract.to_string(),
        }),
        vec![],
    )?);
    let update_membership_admin_submsg = SubMsg::new(wasm_execute(
        membership_contract.to_string(),
        &membership_common_api::msg::ExecuteMsg::UpdateAdmin(
            membership_common_api::api::UpdateAdminMsg {
                new_admin: governance_controller_contract.to_string(),
            },
        ),
        vec![],
    )?);
    let update_council_membership_admin_submsg = SubMsg::new(wasm_execute(
        council_membership_contract.to_string(),
        &membership_common_api::msg::ExecuteMsg::UpdateAdmin(
            membership_common_api::api::UpdateAdminMsg {
                new_admin: governance_controller_contract.to_string(),
            },
        ),
        vec![],
    )?);

    let add_membership_weight_change_hook_submsg = SubMsg::new(wasm_execute(
        membership_contract.to_string(),
        &membership_common_api::msg::ExecuteMsg::AddWeightChangeHook(WeightChangeHookMsg {
            hook_addr: governance_controller_contract.to_string(),
        }),
        vec![],
    )?);
    let add_council_membership_weight_change_hook_submsg = SubMsg::new(wasm_execute(
        council_membership_contract.to_string(),
        &membership_common_api::msg::ExecuteMsg::AddWeightChangeHook(WeightChangeHookMsg {
            hook_addr: governance_controller_contract.to_string(),
        }),
        vec![],
    )?);

    let funds_distributor = FUNDS_DISTRIBUTOR_CONTRACT.load(ctx.deps.storage)?;
    let migrate_funds_distributor = SubMsg::new(Wasm(Migrate {
        contract_addr: funds_distributor.to_string(),
        new_code_id: version_info.funds_distributor_code_id,
        msg: to_binary(&funds_distributor_api::msg::MigrateMsg {
            new_admin: governance_controller_contract.to_string(),
        })?,
    }));

    let enterprise_governance = ENTERPRISE_GOVERNANCE_CONTRACT.load(ctx.deps.storage)?;
    let migrate_enterprise_governance_submsg = SubMsg::new(Wasm(Migrate {
        contract_addr: enterprise_governance.to_string(),
        new_code_id: version_info.enterprise_governance_code_id,
        msg: to_binary(&enterprise_governance_api::msg::MigrateMsg {
            new_admin: governance_controller_contract.to_string(),
        })?,
    }));

    COMPONENT_CONTRACTS.save(
        ctx.deps.storage,
        &ComponentContracts {
            enterprise_governance_contract,
            enterprise_governance_controller_contract: governance_controller_contract,
            enterprise_treasury_contract: treasury_contract,
            funds_distributor_contract,
            membership_contract,
            council_membership_contract,
        },
    )?;

    // remove all the old storage
    NATIVE_ASSET_WHITELIST.clear(ctx.deps.storage);
    CW20_ASSET_WHITELIST.clear(ctx.deps.storage);
    CW1155_ASSET_WHITELIST.clear(ctx.deps.storage);

    NFT_WHITELIST.clear(ctx.deps.storage);

    DAO_GOV_CONFIG.remove(ctx.deps.storage);
    DAO_COUNCIL.remove(ctx.deps.storage);
    DAO_CODE_VERSION.remove(ctx.deps.storage);

    MULTISIG_MEMBERS.clear(ctx.deps.storage);

    DAO_MEMBERSHIP_CONTRACT.remove(ctx.deps.storage);
    ENTERPRISE_GOVERNANCE_CONTRACT.remove(ctx.deps.storage);
    FUNDS_DISTRIBUTOR_CONTRACT.remove(ctx.deps.storage);

    PROPOSAL_INFOS.clear(ctx.deps.storage);

    MIGRATION_INFO.remove(ctx.deps.storage);

    Ok(Response::new()
        .add_submessage(update_treasury_admin_submsg)
        .add_submessage(update_membership_admin_submsg)
        .add_submessage(update_council_membership_admin_submsg)
        .add_submessage(add_membership_weight_change_hook_submsg)
        .add_submessage(add_council_membership_weight_change_hook_submsg)
        .add_submessage(migrate_funds_distributor)
        .add_submessage(migrate_enterprise_governance_submsg))
}
