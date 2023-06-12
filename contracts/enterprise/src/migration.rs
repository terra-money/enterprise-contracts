use crate::contract::{ENTERPRISE_GOVERNANCE_CONTROLLER_REPLY_ID, ENTERPRISE_TREASURY_REPLY_ID};
use crate::state::{ComponentContracts, COMPONENT_CONTRACTS, DAO_TYPE};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{
    wasm_instantiate, Addr, Decimal, DepsMut, Env, Order, StdResult, SubMsg, Uint128,
};
use cw_asset::AssetInfo;
use cw_storage_plus::{Item, Map};
use cw_utils::Duration;
use enterprise_governance_controller_api::api::GovConfig;
use enterprise_protocol::api::DaoType;
use enterprise_protocol::error::DaoResult;
use multisig_membership_api::api::UserWeight;

const NATIVE_ASSET_WHITELIST: Map<String, ()> = Map::new("native_asset_whitelist");
const CW20_ASSET_WHITELIST: Map<Addr, ()> = Map::new("cw20_asset_whitelist");
const CW1155_ASSET_WHITELIST: Map<(Addr, String), ()> = Map::new("cw1155_asset_whitelist");

const NFT_WHITELIST: Map<Addr, ()> = Map::new("nft_whitelist");

const DAO_GOV_CONFIG: Item<DaoGovConfig> = Item::new("dao_gov_config");

const DAO_MEMBERSHIP_CONTRACT: Item<Addr> = Item::new("dao_membership_contract");

const MULTISIG_MEMBERS: Map<Addr, Uint128> = Map::new("multisig_members");

const ENTERPRISE_GOVERNANCE_CONTRACT: Item<Addr> = Item::new("enterprise_governance_contract");
const FUNDS_DISTRIBUTOR_CONTRACT: Item<Addr> = Item::new("funds_distributor_contract");

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
    pub vote_duration: u64, // TODO: change from u64 to Duration
    /// Duration that has to pass for unstaked membership tokens to be claimable
    pub unlocking_period: Duration,
    /// Optional minimum amount of DAO's governance unit to be required to create a deposit.
    pub minimum_deposit: Option<Uint128>,
    /// If set to true, this will allow DAOs to execute proposals that have reached quorum and
    /// threshold, even before their voting period ends.
    pub allow_early_proposal_execution: bool,
}

pub fn migrate_to_rewrite(
    mut deps: DepsMut,
    env: Env,
    treasury_code_id: u64,
    governance_controller_code_id: u64,
    token_membership_code_id: u64,
    nft_membership_code_id: u64,
    multisig_membership_code_id: u64,
) -> DaoResult<Vec<SubMsg>> {
    let enterprise_governance_contract = ENTERPRISE_GOVERNANCE_CONTRACT.load(deps.storage)?;
    let funds_distributor_contract = FUNDS_DISTRIBUTOR_CONTRACT.load(deps.storage)?;
    COMPONENT_CONTRACTS.save(
        deps.storage,
        &ComponentContracts {
            enterprise_governance_contract,
            // TODO: do this through an intermediary structure, this is a really plebish way to accomplish it (with Addr::unchecked)
            enterprise_governance_controller_contract: Addr::unchecked(""),
            enterprise_treasury_contract: Addr::unchecked(""),
            funds_distributor_contract,
            membership_contract: Addr::unchecked(""),
        },
    )?;

    let treasury_submsg = create_treasury_contract(deps.branch(), env.clone(), treasury_code_id)?;

    let governance_controller_submsg =
        create_treasury_contract(deps.branch(), env.clone(), governance_controller_code_id)?;

    let membership_submsg = create_enterprise_membership_contract(
        deps,
        env,
        token_membership_code_id,
        nft_membership_code_id,
        multisig_membership_code_id,
    )?;

    Ok(vec![
        treasury_submsg,
        governance_controller_submsg,
        membership_submsg,
    ])
}

pub fn create_treasury_contract(
    deps: DepsMut,
    env: Env,
    treasury_code_id: u64,
) -> DaoResult<SubMsg> {
    let mut asset_whitelist = NATIVE_ASSET_WHITELIST
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(String, ())>>>()?
        .into_iter()
        .map(|(denom, _)| AssetInfo::native(denom))
        .collect::<Vec<AssetInfo>>();

    let mut cw20_asset_whitelist = CW20_ASSET_WHITELIST
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(Addr, ())>>>()?
        .into_iter()
        .map(|(addr, _)| AssetInfo::cw20(addr))
        .collect::<Vec<AssetInfo>>();

    let mut cw1155_asset_whitelist = CW1155_ASSET_WHITELIST
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<((Addr, String), ())>>>()?
        .into_iter()
        .map(|((addr, token_id), _)| AssetInfo::cw1155(addr, token_id))
        .collect::<Vec<AssetInfo>>();

    asset_whitelist.append(&mut cw20_asset_whitelist);
    asset_whitelist.append(&mut cw1155_asset_whitelist);

    let nft_whitelist = NFT_WHITELIST
        .range(deps.storage, None, None, Ascending)
        .collect::<StdResult<Vec<(Addr, ())>>>()?
        .into_iter()
        .map(|(addr, _)| addr.to_string())
        .collect();

    let submsg = SubMsg::reply_on_success(
        wasm_instantiate(
            treasury_code_id,
            &enterprise_treasury_api::msg::InstantiateMsg {
                enterprise_contract: env.contract.address.to_string(),
                asset_whitelist: Some(asset_whitelist),
                nft_whitelist: Some(nft_whitelist),
            },
            vec![],
            "Enterprise treasury".to_string(),
        )?,
        ENTERPRISE_TREASURY_REPLY_ID,
    );

    Ok(submsg)
}

pub fn create_governance_controller_contract(
    deps: DepsMut,
    env: Env,
    governance_controller_code_id: u64,
) -> DaoResult<SubMsg> {
    let gov_config = DAO_GOV_CONFIG.load(deps.storage)?;

    let submsg = SubMsg::reply_on_success(
        wasm_instantiate(
            governance_controller_code_id,
            &enterprise_governance_controller_api::msg::InstantiateMsg {
                enterprise_contract: env.contract.address.to_string(),
                gov_config: GovConfig {
                    quorum: gov_config.quorum,
                    threshold: gov_config.threshold,
                    veto_threshold: gov_config.veto_threshold,
                    vote_duration: gov_config.vote_duration,
                    unlocking_period: gov_config.unlocking_period, // TODO: do we even need this?
                    minimum_deposit: gov_config.minimum_deposit,
                    allow_early_proposal_execution: gov_config.allow_early_proposal_execution,
                },
            },
            vec![],
            "Enterprise governance controller".to_string(),
        )?,
        ENTERPRISE_GOVERNANCE_CONTROLLER_REPLY_ID,
    );

    Ok(submsg)
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
            wasm_instantiate(
                token_membership_code_id,
                &token_staking_api::msg::InstantiateMsg {
                    admin: env.contract.address.to_string(),
                    token_contract: cw20_contract.to_string(),
                    unlocking_period: gov_config.unlocking_period,
                },
                vec![],
                "Token staking membership".to_string(), // TODO: unify with the name in enterprise factory
            )?
        }
        DaoType::Nft => {
            let cw721_contract = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;
            wasm_instantiate(
                nft_membership_code_id,
                &nft_staking_api::msg::InstantiateMsg {
                    admin: env.contract.address.to_string(),
                    nft_contract: cw721_contract.to_string(),
                    unlocking_period: gov_config.unlocking_period,
                },
                vec![],
                "NFT staking membership".to_string(), // TODO: unify with the name in enterprise factory
            )?
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
            wasm_instantiate(
                multisig_membership_code_id,
                &multisig_membership_api::msg::InstantiateMsg {
                    admin: env.contract.address.to_string(),
                    initial_weights: Some(initial_weights),
                },
                vec![],
                "Multisig membership".to_string(), // TODO: unify with the name in enterprise factory
            )?
        }
    };

    let submsg = SubMsg::reply_on_success(msg, ENTERPRISE_GOVERNANCE_CONTROLLER_REPLY_ID);

    Ok(submsg)
}
