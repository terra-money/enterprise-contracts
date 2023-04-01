use crate::state::{
    CONFIG, DAO_ADDRESSES, DAO_ID_COUNTER, ENTERPRISE_CODE_IDS, GLOBAL_ASSET_WHITELIST,
    GLOBAL_NFT_WHITELIST,
};
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, StdResult, SubMsg, Uint64, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::parse_reply_instantiate_data;
use enterprise_factory_api::api::{
    AllDaosResponse, Config, ConfigResponse, CreateDaoMembershipMsg, CreateDaoMsg,
    EnterpriseCodeIdsMsg, EnterpriseCodeIdsResponse, IsEnterpriseCodeIdMsg,
    IsEnterpriseCodeIdResponse, QueryAllDaosMsg,
};
use enterprise_factory_api::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use enterprise_protocol::api::{
    AssetWhitelistResponse, DaoMembershipInfo, NewDaoMembershipMsg, NewMembershipInfo,
    NftWhitelistResponse,
};
use enterprise_protocol::error::{DaoError, DaoResult};
use itertools::Itertools;
use CreateDaoMembershipMsg::{ExistingMembership, NewMembership};
use NewMembershipInfo::{NewMultisig, NewNft, NewToken};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_QUERY_LIMIT: u32 = 100;
const DEFAULT_QUERY_LIMIT: u32 = 50;

pub const ENTERPRISE_INSTANTIATE_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> DaoResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &msg.config)?;
    GLOBAL_ASSET_WHITELIST.save(
        deps.storage,
        &msg.global_asset_whitelist.unwrap_or_default(),
    )?;
    GLOBAL_NFT_WHITELIST.save(deps.storage, &msg.global_nft_whitelist.unwrap_or_default())?;
    DAO_ID_COUNTER.save(deps.storage, &1u64)?;

    ENTERPRISE_CODE_IDS.save(deps.storage, msg.config.enterprise_code_id, &())?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> DaoResult<Response> {
    match msg {
        ExecuteMsg::CreateDao(msg) => create_dao(deps, env, msg),
    }
}

fn create_dao(deps: DepsMut, env: Env, msg: CreateDaoMsg) -> DaoResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    let dao_membership_info = match msg.dao_membership {
        NewMembership(info) => {
            let membership_contract_code_id = match info {
                NewToken(_) => config.cw20_code_id,
                NewNft(_) => config.cw721_code_id,
                NewMultisig(_) => config.cw3_fixed_multisig_code_id,
            };
            DaoMembershipInfo::New(NewDaoMembershipMsg {
                membership_contract_code_id,
                membership_info: info,
            })
        }
        ExistingMembership(info) => DaoMembershipInfo::Existing(info),
    };

    let instantiate_enterprise_msg = enterprise_protocol::msg::InstantiateMsg {
        enterprise_governance_code_id: config.enterprise_governance_code_id,
        funds_distributor_code_id: config.funds_distributor_code_id,
        dao_metadata: msg.dao_metadata.clone(),
        dao_gov_config: msg.dao_gov_config,
        dao_council: msg.dao_council,
        dao_membership_info,
        enterprise_factory_contract: env.contract.address.to_string(),
        asset_whitelist: msg.asset_whitelist,
        nft_whitelist: msg.nft_whitelist,
        minimum_weight_for_rewards: msg.minimum_weight_for_rewards,
    };
    let create_dao_submsg = SubMsg::reply_on_success(
        WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: config.enterprise_code_id,
            msg: to_binary(&instantiate_enterprise_msg)?,
            funds: vec![],
            label: msg.dao_metadata.name,
        },
        ENTERPRISE_INSTANTIATE_ID,
    );

    Ok(Response::new()
        .add_attribute("action", "create_dao")
        .add_submessage(create_dao_submsg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> DaoResult<Response> {
    match msg.id {
        ENTERPRISE_INSTANTIATE_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let addr = deps.api.addr_validate(&contract_address)?;

            let id = DAO_ID_COUNTER.load(deps.storage)?;
            let next_id = id + 1;
            DAO_ID_COUNTER.save(deps.storage, &next_id)?;

            DAO_ADDRESSES.save(deps.storage, id, &addr)?;

            // Update the admin of the DAO contract to be the DAO itself
            let update_admin_msg = SubMsg::new(WasmMsg::UpdateAdmin {
                contract_addr: contract_address.clone(),
                admin: contract_address.clone(),
            });

            Ok(Response::new()
                .add_submessage(update_admin_msg)
                .add_attribute("action", "instantiate_dao")
                .add_attribute("dao_address", contract_address))
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
        QueryMsg::GlobalAssetWhitelist {} => to_binary(&query_asset_whitelist(deps)?)?,
        QueryMsg::GlobalNftWhitelist {} => to_binary(&query_nft_whitelist(deps)?)?,
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

pub fn query_asset_whitelist(deps: Deps) -> DaoResult<AssetWhitelistResponse> {
    let assets = GLOBAL_ASSET_WHITELIST.load(deps.storage)?;

    Ok(AssetWhitelistResponse { assets })
}

pub fn query_nft_whitelist(deps: Deps) -> DaoResult<NftWhitelistResponse> {
    let nfts = GLOBAL_NFT_WHITELIST.load(deps.storage)?;

    Ok(NftWhitelistResponse { nfts })
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
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> DaoResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    CONFIG.save(
        deps.storage,
        &Config {
            enterprise_code_id: msg.new_enterprise_code_id,
            enterprise_governance_code_id: msg.new_enterprise_governance_code_id,
            funds_distributor_code_id: msg.new_funds_distributor_code_id,
            ..config
        },
    )?;

    ENTERPRISE_CODE_IDS.save(deps.storage, msg.new_enterprise_code_id, &())?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new())
}
