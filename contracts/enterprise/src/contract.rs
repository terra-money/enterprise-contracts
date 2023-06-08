use crate::state::{
    DAO_CODE_VERSION, DAO_CREATION_DATE, DAO_MEMBERSHIP_CONTRACT, DAO_METADATA, DAO_TYPE,
    ENTERPRISE_FACTORY_CONTRACT, ENTERPRISE_GOVERNANCE_CONTRACT, FUNDS_DISTRIBUTOR_CONTRACT,
};
use crate::validate::validate_existing_dao_contract;
use common::commons::ModifyValue::Change;
use common::cw::{Context, QueryContext};
use cosmwasm_std::{
    entry_point, to_binary, wasm_instantiate, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Reply, Response, StdError, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20Coin, Logo, MinterResponse};
use cw_utils::parse_reply_instantiate_data;
use enterprise_protocol::api::DaoType::{Multisig, Nft};
use enterprise_protocol::api::{
    DaoInfoResponse, DaoMembershipInfo, DaoType, ExistingDaoMembershipMsg, NewDaoMembershipMsg,
    NewMembershipInfo, NewMultisigMembershipInfo, NewNftMembershipInfo, NewTokenMembershipInfo,
    UpdateMetadataMsg, UpgradeDaoMsg,
};
use enterprise_protocol::error::DaoError::{Std, ZeroInitialDaoBalance};
use enterprise_protocol::error::{DaoError, DaoResult};
use enterprise_protocol::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use funds_distributor_api::api::UserWeight;
use std::ops::Add;
use cosmwasm_std::CosmosMsg::Wasm;
use DaoError::ZeroInitialWeightMember;
use DaoMembershipInfo::{Existing, New};
use DaoType::Token;
use NewMembershipInfo::{NewMultisig, NewNft, NewToken};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:enterprise";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DAO_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 1;
pub const ENTERPRISE_GOVERNANCE_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 2;
pub const FUNDS_DISTRIBUTOR_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 3;

pub const CODE_VERSION: u8 = 5;

pub const DEFAULT_QUERY_LIMIT: u8 = 50;
pub const MAX_QUERY_LIMIT: u8 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> DaoResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    DAO_CREATION_DATE.save(deps.storage, &env.block.time)?;

    DAO_METADATA.save(deps.storage, &msg.dao_metadata)?;
    ENTERPRISE_FACTORY_CONTRACT.save(
        deps.storage,
        &deps.api.addr_validate(&msg.enterprise_factory_contract)?,
    )?;
    DAO_CODE_VERSION.save(deps.storage, &CODE_VERSION.into())?;

    // instantiate the governance contract
    let instantiate_governance_contract_submsg = SubMsg::reply_on_success(
        Wasm(WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.enterprise_governance_code_id,
            msg: to_binary(&enterprise_governance_api::msg::InstantiateMsg {
                enterprise_contract: env.contract.address.to_string(),
            })?,
            funds: vec![],
            label: "Governance contract".to_string(),
        }),
        ENTERPRISE_GOVERNANCE_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    let ctx = Context { deps, env, info };

    let mut submessages = match msg.dao_membership_info {
        New(membership) => instantiate_new_membership_dao(
            ctx,
            membership,
            msg.funds_distributor_code_id,
            msg.minimum_weight_for_rewards,
        )?,
        Existing(membership) => instantiate_existing_membership_dao(
            ctx,
            membership,
            msg.funds_distributor_code_id,
            msg.minimum_weight_for_rewards,
        )?,
    };

    submessages.push(instantiate_governance_contract_submsg);

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_submessages(submessages))
}

fn instantiate_funds_distributor_submsg(
    ctx: &Context,
    funds_distributor_code_id: u64,
    minimum_weight_for_rewards: Option<Uint128>,
    initial_weights: Vec<UserWeight>,
) -> DaoResult<SubMsg> {
    let instantiate_funds_distributor_contract_submsg = SubMsg::reply_on_success(
        Wasm(WasmMsg::Instantiate {
            admin: Some(ctx.env.contract.address.to_string()),
            code_id: funds_distributor_code_id,
            msg: to_binary(&funds_distributor_api::msg::InstantiateMsg {
                enterprise_contract: ctx.env.contract.address.to_string(),
                initial_weights,
                minimum_eligible_weight: minimum_weight_for_rewards,
            })?,
            funds: vec![],
            label: "Funds distributor contract".to_string(),
        }),
        FUNDS_DISTRIBUTOR_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    Ok(instantiate_funds_distributor_contract_submsg)
}

fn instantiate_new_membership_dao(
    ctx: Context,
    membership: NewDaoMembershipMsg,
    funds_distributor_code_id: u64,
    minimum_weight_for_rewards: Option<Uint128>,
) -> DaoResult<Vec<SubMsg>> {
    match membership.membership_info {
        NewToken(info) => instantiate_new_token_dao(
            ctx,
            *info,
            membership.membership_contract_code_id,
            funds_distributor_code_id,
            minimum_weight_for_rewards,
        ),
        NewNft(info) => instantiate_new_nft_dao(
            ctx,
            info,
            membership.membership_contract_code_id,
            funds_distributor_code_id,
            minimum_weight_for_rewards,
        ),
        NewMultisig(info) => instantiate_new_multisig_dao(
            ctx,
            info,
            funds_distributor_code_id,
            minimum_weight_for_rewards,
        ),
    }
}

fn instantiate_new_token_dao(
    ctx: Context,
    info: NewTokenMembershipInfo,
    cw20_code_id: u64,
    funds_distributor_code_id: u64,
    minimum_weight_for_rewards: Option<Uint128>,
) -> DaoResult<Vec<SubMsg>> {
    if let Some(initial_dao_balance) = info.initial_dao_balance {
        if initial_dao_balance == Uint128::zero() {
            return Err(ZeroInitialDaoBalance);
        }
    }

    for initial_balance in info.initial_token_balances.iter() {
        if initial_balance.amount == Uint128::zero() {
            return Err(ZeroInitialWeightMember);
        }
    }

    DAO_TYPE.save(ctx.deps.storage, &Token)?;

    let marketing = info
        .token_marketing
        .map(|marketing| cw20_base::msg::InstantiateMarketingInfo {
            project: marketing.project,
            description: marketing.description,
            marketing: marketing
                .marketing_owner
                .or_else(|| Some(ctx.env.contract.address.to_string())),
            logo: marketing.logo_url.map(Logo::Url),
        })
        .or_else(|| {
            Some(cw20_base::msg::InstantiateMarketingInfo {
                project: None,
                description: None,
                marketing: Some(ctx.env.contract.address.to_string()),
                logo: None,
            })
        });

    let initial_balances = match info.initial_dao_balance {
        None => info.initial_token_balances,
        Some(initial_dao_balance) => {
            let mut token_balances = info.initial_token_balances;
            token_balances.push(Cw20Coin {
                address: ctx.env.contract.address.to_string(),
                amount: initial_dao_balance,
            });
            token_balances
        }
    };

    let create_token_msg = cw20_base::msg::InstantiateMsg {
        name: info.token_name.clone(),
        symbol: info.token_symbol,
        decimals: info.token_decimals,
        initial_balances,
        mint: info.token_mint.or_else(|| {
            Some(MinterResponse {
                minter: ctx.env.contract.address.to_string(),
                cap: None,
            })
        }),
        marketing,
    };

    let instantiate_dao_token_submsg = SubMsg::reply_on_success(
        wasm_instantiate(cw20_code_id, &create_token_msg, vec![], info.token_name)?,
        DAO_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    Ok(vec![
        instantiate_dao_token_submsg,
        instantiate_funds_distributor_submsg(
            &ctx,
            funds_distributor_code_id,
            minimum_weight_for_rewards,
            vec![],
        )?,
    ])
}

fn instantiate_new_multisig_dao(
    ctx: Context,
    info: NewMultisigMembershipInfo,
    funds_distributor_code_id: u64,
    minimum_weight_for_rewards: Option<Uint128>,
) -> DaoResult<Vec<SubMsg>> {
    DAO_TYPE.save(ctx.deps.storage, &Multisig)?;

    let mut total_weight = Uint128::zero();

    let mut initial_weights: Vec<UserWeight> = vec![];

    for member in info.multisig_members.into_iter() {
        if member.weight == Uint128::zero() {
            return Err(ZeroInitialWeightMember);
        }

        initial_weights.push(UserWeight {
            user: member.address,
            weight: member.weight,
        });

        total_weight = total_weight.add(member.weight);
    }

    DAO_MEMBERSHIP_CONTRACT.save(ctx.deps.storage, &ctx.env.contract.address)?;

    Ok(vec![instantiate_funds_distributor_submsg(
        &ctx,
        funds_distributor_code_id,
        minimum_weight_for_rewards,
        initial_weights,
    )?])
}

fn instantiate_new_nft_dao(
    ctx: Context,
    info: NewNftMembershipInfo,
    cw721_code_id: u64,
    funds_distributor_code_id: u64,
    minimum_weight_for_rewards: Option<Uint128>,
) -> DaoResult<Vec<SubMsg>> {
    DAO_TYPE.save(ctx.deps.storage, &Nft)?;

    let minter = match info.minter {
        None => ctx.env.contract.address.to_string(),
        Some(minter) => minter,
    };
    let instantiate_msg = cw721_base::msg::InstantiateMsg {
        name: info.nft_name,
        symbol: info.nft_symbol,
        minter,
    };
    let submsg = SubMsg::reply_on_success(
        wasm_instantiate(
            cw721_code_id,
            &instantiate_msg,
            vec![],
            "DAO NFT".to_string(),
        )?,
        DAO_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
    );
    Ok(vec![
        submsg,
        instantiate_funds_distributor_submsg(
            &ctx,
            funds_distributor_code_id,
            minimum_weight_for_rewards,
            vec![],
        )?,
    ])
}

fn instantiate_existing_membership_dao(
    ctx: Context,
    membership: ExistingDaoMembershipMsg,
    funds_distributor_code_id: u64,
    minimum_weight_for_rewards: Option<Uint128>,
) -> DaoResult<Vec<SubMsg>> {
    let membership_addr = ctx
        .deps
        .api
        .addr_validate(&membership.membership_contract_addr)?;

    validate_existing_dao_contract(
        &ctx,
        &membership.dao_type,
        &membership.membership_contract_addr,
    )?;

    DAO_TYPE.save(ctx.deps.storage, &membership.dao_type)?;

    let mut initial_weights: Vec<UserWeight> = vec![];

    match membership.dao_type {
        Token | Nft => {
            DAO_MEMBERSHIP_CONTRACT.save(ctx.deps.storage, &membership_addr)?;
        }
        Multisig => {
            DAO_MEMBERSHIP_CONTRACT.save(ctx.deps.storage, &ctx.env.contract.address)?;

            // TODO: gotta do an integration test for this
            let mut total_weight = Uint128::zero();
            let mut last_voter: Option<String> = None;
            while {
                let query_msg = cw3::Cw3QueryMsg::ListVoters {
                    start_after: last_voter.clone(),
                    limit: None,
                };

                last_voter = None;

                let voters: cw3::VoterListResponse = ctx
                    .deps
                    .querier
                    .query_wasm_smart(&membership.membership_contract_addr, &query_msg)?;

                for voter in voters.voters {
                    last_voter = Some(voter.addr.clone());

                    initial_weights.push(UserWeight {
                        user: voter.addr,
                        weight: voter.weight.into(),
                    });

                    total_weight = total_weight.add(Uint128::from(voter.weight));
                }

                last_voter.is_some()
            } {}
        }
    }

    Ok(vec![instantiate_funds_distributor_submsg(
        &ctx,
        funds_distributor_code_id,
        minimum_weight_for_rewards,
        initial_weights,
    )?])
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> DaoResult<Response> {
    let ctx = &mut Context { deps, env, info };

    match msg {
        ExecuteMsg::UpdateMetadata(msg) => update_metadata(ctx, msg),
    }
}

fn update_metadata(ctx: &mut Context, msg: UpdateMetadataMsg) -> DaoResult<Response> {
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

    Ok(Response::new().add_attribute("action", "update_metadata"))
}

// TODO: there should be a message, and then this should dispatch to other contracts
fn _upgrade_dao(env: Env, msg: UpgradeDaoMsg) -> DaoResult<Vec<SubMsg>> {
    Ok(vec![SubMsg::new(WasmMsg::Migrate {
        contract_addr: env.contract.address.to_string(),
        new_code_id: msg.new_dao_code_id,
        msg: msg.migrate_msg,
    })])
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> DaoResult<Response> {
    match msg.id {
        DAO_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let addr = deps.api.addr_validate(&contract_address)?;

            DAO_MEMBERSHIP_CONTRACT.save(deps.storage, &addr)?;

            Ok(Response::new())
        }
        ENTERPRISE_GOVERNANCE_CONTRACT_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;
            let addr = deps.api.addr_validate(&contract_address)?;

            ENTERPRISE_GOVERNANCE_CONTRACT.save(deps.storage, &addr)?;

            Ok(Response::new().add_attribute("governance_contract", addr.to_string()))
        }
        FUNDS_DISTRIBUTOR_CONTRACT_INSTANTIATE_REPLY_ID => {
            let contract_address = parse_reply_instantiate_data(msg)
                .map_err(|_| StdError::generic_err("error parsing instantiate reply"))?
                .contract_address;

            let addr = deps.api.addr_validate(&contract_address)?;

            FUNDS_DISTRIBUTOR_CONTRACT.save(deps.storage, &addr)?;

            Ok(Response::new().add_attribute("funds_distributor_contract", addr.to_string()))
        }
        _ => Err(Std(StdError::generic_err("No such reply ID found"))),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> DaoResult<Binary> {
    let qctx = QueryContext::from(deps, env);

    let response = match msg {
        QueryMsg::DaoInfo {} => to_binary(&query_dao_info(qctx)?)?,
    };
    Ok(response)
}

pub fn query_dao_info(qctx: QueryContext) -> DaoResult<DaoInfoResponse> {
    // TODO: some stuff (like gov config) shouldn't live here anymore
    let creation_date = DAO_CREATION_DATE.load(qctx.deps.storage)?;
    let metadata = DAO_METADATA.load(qctx.deps.storage)?;
    let dao_type = DAO_TYPE.load(qctx.deps.storage)?;
    let dao_membership_contract = DAO_MEMBERSHIP_CONTRACT.load(qctx.deps.storage)?;
    let enterprise_factory_contract = ENTERPRISE_FACTORY_CONTRACT.load(qctx.deps.storage)?;
    let funds_distributor_contract = FUNDS_DISTRIBUTOR_CONTRACT.load(qctx.deps.storage)?;
    let dao_code_version = DAO_CODE_VERSION.load(qctx.deps.storage)?;

    Ok(DaoInfoResponse {
        creation_date,
        metadata,
        dao_type,
        dao_membership_contract,
        enterprise_factory_contract,
        funds_distributor_contract,
        dao_code_version,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> DaoResult<Response> {
    // TODO: if version < 5, either fail or migrate to version 5 first

    Ok(Response::new().add_attribute("action", "migrate"))
}
