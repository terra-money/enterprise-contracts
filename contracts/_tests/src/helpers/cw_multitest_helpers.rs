use std::collections::HashMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Addr, Coin, Uint128, Uint64};
use cw20::Cw20Coin;
use cw_asset::{AssetBase, AssetInfoBase};
use cw_multi_test::{App, AppResponse, ContractWrapper, Executor};
use cw_utils::Duration;
use denom_staking_api::api::DenomConfigResponse;
use enterprise_factory_api::api::{
    AllDaosResponse, Config, CreateDaoMembershipMsg, CreateDaoMsg, NewCw20MembershipMsg,
};
use enterprise_governance_controller_api::api::{
    CastVoteMsg, CreateProposalMsg, DaoCouncilSpec, ExecuteProposalMsg, GovConfig, ProposalAction,
    ProposalActionType, ProposalParams, ProposalResponse, ProposalsParams, ProposalsResponse,
};
use enterprise_protocol::api::{
    ComponentContractsResponse, DaoInfoResponse, DaoMetadata, DaoSocialData, DaoType, Logo,
};
use enterprise_versioning_api::api::{AddVersionMsg, Version, VersionInfo};

use membership_common_api::api::{
    TotalWeightParams, TotalWeightResponse, UserWeightParams, UserWeightResponse,
};
use nft_staking_api::api::NftConfigResponse;
use poll_engine_api::api::VoteOutcome;

use token_staking_api::api::{ClaimMsg, TokenConfigResponse, UnstakeMsg};

use crate::traits::{IntoAddr, IntoDecimal, IntoExecuteMsg, IntoUint};

pub type AppResult = Result<AppResponse, anyhow::Error>;

pub const ADMIN: &str = "admin";

pub const USER_DAO_CREATOR: &str = "dao_creator";

pub const USER1: &str = "user1";
pub const USER2: &str = "user2";
pub const USER3: &str = "user3";

pub const ULUNA: &str = "uluna";

pub const CW20_TOKEN1: &str = "token_addr1";
pub const CW20_TOKEN2: &str = "token_addr2";

pub const NFT_TOKEN1: &str = "nft_token_addr1";
pub const NFT_TOKEN2: &str = "nft_token_addr2";

pub const CODE_ID_ATTESTATION: u64 = 1;
pub const CODE_ID_GOV_CONTROLLER: u64 = 2;
pub const CODE_ID_FUNDS_DISTRIBUTOR: u64 = 3;
pub const CODE_ID_ENTERPRISE: u64 = 4;
pub const CODE_ID_FACADE: u64 = 5;
pub const CODE_ID_FACADE_V1: u64 = 6;
pub const CODE_ID_FACADE_V2: u64 = 7;
pub const CODE_ID_FACTORY: u64 = 8;
pub const CODE_ID_GOVERNANCE: u64 = 9;
pub const CODE_ID_OUTPOSTS: u64 = 10;
pub const CODE_ID_MEMBERSHIP_DENOM: u64 = 11;
pub const CODE_ID_MEMBERSHIP_NFT: u64 = 12;
pub const CODE_ID_MEMBERSHIP_MULTISIG: u64 = 13;
pub const CODE_ID_MEMBERSHIP_TOKEN: u64 = 14;
pub const CODE_ID_TREASURY: u64 = 15;
pub const CODE_ID_VERSIONING: u64 = 16;
pub const CODE_ID_CW3: u64 = 17;
pub const CODE_ID_CW20: u64 = 18;
pub const CODE_ID_CW721: u64 = 19;

pub const ADDR_VERSIONING: &str = "contract0";
pub const ADDR_FACTORY: &str = "contract1";
pub const ADDR_FACADE_V1: &str = "contract2";
pub const ADDR_FACADE_V2: &str = "contract3";
pub const ADDR_FACADE: &str = "contract4";

pub const DAO_NAME: &str = "my_dao";
pub const DAO_TOKEN: &str = "dao_token";

pub const COUNCIL: [&str; 3] = ["council_1", "council_2", "council_3"];

pub const MINIMUM_DEPOSIT: u128 = 10;

pub const INITIAL_DAO_BALANCE: u128 = 1_000_000;

pub const UNLOCKING_PERIOD_GOV: u64 = 60 * 60 * 24 * 7;
pub const UNLOCKING_PERIOD_MEMBERSHIP: u64 = 60 * 60 * 24 * 7;
pub const VOTE_DURATION: u64 = 60 * 60 * 24 * 7;

pub const USERS: [&str; 5] = ["user_1", "user_2", "user_3", "user_4", "user_5"];

pub const INITIAL_USERS_BALANCE_EACH: u128 = 1_000;

#[cw_serde]
pub struct Addresses {
    pub enterprise: Addr,
    pub others: ComponentContractsResponse,
}

pub fn startup() -> App {
    let mut app = App::default();

    let code_id_attestation = app.store_code(Box::new(
        ContractWrapper::new(
            attestation::contract::execute,
            attestation::contract::instantiate,
            attestation::contract::query,
        )
        .with_reply(attestation::contract::reply),
    ));

    assert_eq!(CODE_ID_ATTESTATION, code_id_attestation);

    let code_id_controller = app.store_code(Box::new(
        ContractWrapper::new(
            enterprise_governance_controller::contract::execute,
            enterprise_governance_controller::contract::instantiate,
            enterprise_governance_controller::contract::query,
        )
        .with_reply(enterprise_governance_controller::contract::reply),
    ));

    assert_eq!(CODE_ID_GOV_CONTROLLER, code_id_controller);

    let code_id_funds_distributor = app.store_code(Box::new(
        ContractWrapper::new(
            funds_distributor::contract::execute,
            funds_distributor::contract::instantiate,
            funds_distributor::contract::query,
        )
        .with_reply(funds_distributor::contract::reply),
    ));

    assert_eq!(CODE_ID_FUNDS_DISTRIBUTOR, code_id_funds_distributor);

    let code_id_enterprise = app.store_code(Box::new(
        ContractWrapper::new(
            enterprise::contract::execute,
            enterprise::contract::instantiate,
            enterprise::contract::query,
        )
        .with_reply(enterprise::contract::reply),
    ));

    assert_eq!(CODE_ID_ENTERPRISE, code_id_enterprise);

    let code_id_facade = app.store_code(Box::new(
        ContractWrapper::new(
            enterprise_facade::contract::execute,
            enterprise_facade::contract::instantiate,
            enterprise_facade::contract::query,
        )
        .with_reply(enterprise_facade::contract::reply),
    ));

    assert_eq!(CODE_ID_FACADE, code_id_facade);

    let code_id_facade_v1 = app.store_code(Box::new(
        ContractWrapper::new(
            enterprise_facade_v1::contract::execute,
            enterprise_facade_v1::contract::instantiate,
            enterprise_facade_v1::contract::query,
        )
        .with_reply(enterprise_facade_v1::contract::reply),
    ));

    assert_eq!(CODE_ID_FACADE_V1, code_id_facade_v1);

    let code_id_facade_v2 = app.store_code(Box::new(
        ContractWrapper::new(
            enterprise_facade_v2::contract::execute,
            enterprise_facade_v2::contract::instantiate,
            enterprise_facade_v2::contract::query,
        )
        .with_reply(enterprise_facade_v2::contract::reply),
    ));

    assert_eq!(CODE_ID_FACADE_V2, code_id_facade_v2);

    let code_id_factory = app.store_code(Box::new(
        ContractWrapper::new(
            enterprise_factory::contract::execute,
            enterprise_factory::contract::instantiate,
            enterprise_factory::contract::query,
        )
        .with_reply(enterprise_factory::contract::reply),
    ));

    assert_eq!(CODE_ID_FACTORY, code_id_factory);

    let code_id_governance = app.store_code(Box::new(
        ContractWrapper::new(
            enterprise_governance::contract::execute,
            enterprise_governance::contract::instantiate,
            enterprise_governance::contract::query,
        )
        .with_reply(enterprise_governance::contract::reply),
    ));

    assert_eq!(CODE_ID_GOVERNANCE, code_id_governance);

    let code_id_outposts = app.store_code(Box::new(
        ContractWrapper::new(
            enterprise_outposts::contract::execute,
            enterprise_outposts::contract::instantiate,
            enterprise_outposts::contract::query,
        )
        .with_reply(enterprise_outposts::contract::reply),
    ));

    assert_eq!(CODE_ID_OUTPOSTS, code_id_outposts);

    let code_id_membership_denom = app.store_code(Box::new(
        ContractWrapper::new(
            denom_staking_membership::contract::execute,
            denom_staking_membership::contract::instantiate,
            denom_staking_membership::contract::query,
        )
        .with_reply(denom_staking_membership::contract::reply),
    ));

    assert_eq!(CODE_ID_MEMBERSHIP_DENOM, code_id_membership_denom);

    let code_id_membership_nft = app.store_code(Box::new(
        ContractWrapper::new(
            nft_staking_membership::contract::execute,
            nft_staking_membership::contract::instantiate,
            nft_staking_membership::contract::query,
        )
        .with_reply(nft_staking_membership::contract::reply),
    ));

    assert_eq!(CODE_ID_MEMBERSHIP_NFT, code_id_membership_nft);

    let code_id_membership_multisig = app.store_code(Box::new(
        ContractWrapper::new(
            multisig_membership::contract::execute,
            multisig_membership::contract::instantiate,
            multisig_membership::contract::query,
        )
        .with_reply(multisig_membership::contract::reply),
    ));

    assert_eq!(CODE_ID_MEMBERSHIP_MULTISIG, code_id_membership_multisig);

    let code_id_membership_token = app.store_code(Box::new(
        ContractWrapper::new(
            token_staking_membership::contract::execute,
            token_staking_membership::contract::instantiate,
            token_staking_membership::contract::query,
        )
        .with_reply(token_staking_membership::contract::reply),
    ));

    assert_eq!(CODE_ID_MEMBERSHIP_TOKEN, code_id_membership_token);

    let code_id_treasury = app.store_code(Box::new(
        ContractWrapper::new(
            enterprise_treasury::contract::execute,
            enterprise_treasury::contract::instantiate,
            enterprise_treasury::contract::query,
        )
        .with_reply(enterprise_treasury::contract::reply),
    ));

    assert_eq!(CODE_ID_TREASURY, code_id_treasury);

    let code_id_versioning = app.store_code(Box::new(
        ContractWrapper::new(
            enterprise_versioning::contract::execute,
            enterprise_versioning::contract::instantiate,
            enterprise_versioning::contract::query,
        )
        .with_reply(enterprise_versioning::contract::reply),
    ));

    assert_eq!(CODE_ID_VERSIONING, code_id_versioning);

    let code_id_cw3 = app.store_code(Box::new(ContractWrapper::new(
        cw3_fixed_multisig::contract::execute,
        cw3_fixed_multisig::contract::instantiate,
        cw3_fixed_multisig::contract::query,
    )));

    assert_eq!(code_id_cw3, CODE_ID_CW3);

    let code_id_cw20 = app.store_code(Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    )));

    assert_eq!(code_id_cw20, CODE_ID_CW20);

    let code_id_cw721 = app.store_code(Box::new(ContractWrapper::new(
        cw721_metadata_onchain::entry::execute,
        cw721_metadata_onchain::entry::instantiate,
        cw721_metadata_onchain::entry::query,
    )));

    assert_eq!(code_id_cw721, CODE_ID_CW721);

    app
}

pub fn startup_with_versioning() -> App {
    let mut app = startup();

    let msg: enterprise_versioning_api::msg::InstantiateMsg =
        enterprise_versioning_api::msg::InstantiateMsg {
            admin: ADMIN.to_string(),
        };

    assert_eq!(
        app.instantiate_contract(
            CODE_ID_VERSIONING,
            ADMIN.into_addr(),
            &msg,
            &[],
            "versioning",
            Some(ADMIN.to_string()),
        )
        .unwrap(),
        ADDR_VERSIONING
    );

    let msg = enterprise_versioning_api::msg::ExecuteMsg::AddVersion(AddVersionMsg {
        version: VersionInfo {
            version: Version {
                major: 0,
                minor: 1,
                patch: 0,
            },
            changelog: vec![],
            attestation_code_id: CODE_ID_ATTESTATION,
            enterprise_code_id: CODE_ID_ENTERPRISE,
            enterprise_governance_code_id: CODE_ID_GOVERNANCE,
            enterprise_governance_controller_code_id: CODE_ID_GOV_CONTROLLER,
            enterprise_outposts_code_id: CODE_ID_OUTPOSTS,
            enterprise_treasury_code_id: CODE_ID_TREASURY,
            funds_distributor_code_id: CODE_ID_FUNDS_DISTRIBUTOR,
            token_staking_membership_code_id: CODE_ID_MEMBERSHIP_TOKEN,
            denom_staking_membership_code_id: CODE_ID_MEMBERSHIP_DENOM,
            nft_staking_membership_code_id: CODE_ID_MEMBERSHIP_NFT,
            multisig_membership_code_id: CODE_ID_MEMBERSHIP_MULTISIG,
        },
    });

    app.execute_contract(ADMIN.into_addr(), ADDR_VERSIONING.into_addr(), &msg, &[])
        .unwrap();

    let msg = enterprise_factory_api::msg::InstantiateMsg {
        config: Config {
            admin: ADMIN.into_addr(),
            enterprise_versioning: ADDR_VERSIONING.into_addr(),
            cw20_code_id: CODE_ID_CW20,
            cw721_code_id: CODE_ID_CW721,
        },
        global_asset_whitelist: None,
        global_nft_whitelist: None,
    };

    let contract_factory = app
        .instantiate_contract(
            CODE_ID_FACTORY,
            ADMIN.into_addr(),
            &msg,
            &[],
            "factory",
            Some(ADMIN.to_string()),
        )
        .unwrap();

    assert_eq!(contract_factory, ADDR_FACTORY);

    let facade_v1 = app
        .instantiate_contract(
            CODE_ID_FACADE_V1,
            ADMIN.into_addr(),
            &enterprise_facade_v1::msg::InstantiateMsg {
                enterprise_versioning: ADDR_VERSIONING.to_string(),
            },
            &[],
            "facade v1",
            Some(ADMIN.to_string()),
        )
        .unwrap();

    assert_eq!(facade_v1, ADDR_FACADE_V1);

    let facade_v2 = app
        .instantiate_contract(
            CODE_ID_FACADE_V2,
            ADMIN.into_addr(),
            &enterprise_facade_v2::msg::InstantiateMsg {},
            &[],
            "facade v2",
            Some(ADMIN.to_string()),
        )
        .unwrap();

    assert_eq!(facade_v2, ADDR_FACADE_V2);

    let facade = app
        .instantiate_contract(
            CODE_ID_FACADE,
            ADMIN.into_addr(),
            &enterprise_facade_api::msg::InstantiateMsg {
                enterprise_facade_v1: ADDR_FACADE_V1.to_string(),
                enterprise_facade_v2: ADDR_FACADE_V2.to_string(),
            },
            &[],
            "facade",
            Some(ADMIN.to_string()),
        )
        .unwrap();

    assert_eq!(facade, ADDR_FACADE);

    app
}

pub fn startup_default_dao() -> App {
    let mut app = startup_with_versioning();

    let msg = create_standard_msg_new_dao().into_execute_msg();

    app.execute_contract(
        USER_DAO_CREATOR.into_addr(),
        ADDR_FACTORY.into_addr(),
        &msg,
        &[],
    )
    .unwrap();

    app
}

pub fn startup_custom_dao(dao_info: CreateDaoMsg) -> App {
    let mut app = startup_with_versioning();

    app.execute_contract(
        USER_DAO_CREATOR.into_addr(),
        ADDR_FACTORY.into_addr(),
        &dao_info.into_execute_msg(),
        &[],
    )
    .unwrap();

    app
}

pub fn create_standard_msg_new_dao() -> CreateDaoMsg {
    CreateDaoMsg {
        dao_metadata: DaoMetadata {
            name: DAO_NAME.to_string(),
            description: Some(format!("{DAO_NAME}_description")),
            logo: Logo::Url(format!("{DAO_NAME}_logo_url")),
            socials: DaoSocialData {
                github_username: Some(format!("{DAO_NAME}_github")),
                discord_username: Some(format!("{DAO_NAME}_discord")),
                twitter_username: Some(format!("{DAO_NAME}_twitter")),
                telegram_username: Some(format!("{DAO_NAME}_telegram")),
            },
        },
        gov_config: GovConfig {
            quorum: "0.3".into_decimal(),
            threshold: "0.5".into_decimal(),
            veto_threshold: Some("0.2".into_decimal()),
            vote_duration: VOTE_DURATION,
            minimum_deposit: Some(MINIMUM_DEPOSIT.into()),
            allow_early_proposal_execution: false,
        },
        dao_council: Some(DaoCouncilSpec {
            members: COUNCIL
                .into_iter()
                .map(|member| member.to_string())
                .collect(),
            quorum: "0.5".into_decimal(),
            threshold: "0.5".into_decimal(),
            allowed_proposal_action_types: Some(vec![
                ProposalActionType::UpdateMetadata,
                ProposalActionType::UpgradeDao,
            ]),
        }),
        dao_membership: CreateDaoMembershipMsg::NewCw20(Box::new(NewCw20MembershipMsg {
            token_name: DAO_TOKEN.to_string(),
            token_symbol: DAO_TOKEN.to_uppercase().replace("_", ""),
            token_decimals: 6,
            initial_token_balances: USERS
                .into_iter()
                .map(|name| Cw20Coin {
                    address: name.to_string(),
                    amount: INITIAL_USERS_BALANCE_EACH.into(),
                })
                .collect(),
            initial_dao_balance: Some(INITIAL_DAO_BALANCE.into()),
            token_mint: None, // This will be dao?
            token_marketing: None,
            unlocking_period: Duration::Time(UNLOCKING_PERIOD_MEMBERSHIP),
        })),
        asset_whitelist: None,
        nft_whitelist: None,
        minimum_weight_for_rewards: None,
        proposals_tracked_for_participation_rewards: None,
        cross_chain_treasuries: None,
    }
}

pub fn increase_time_block(app: &mut App, second: u64) {
    app.update_block(|block_info| block_info.time = block_info.time.plus_seconds(second))
}

// QUERIES

pub fn qy_get_all_dao(app: &App) -> HashMap<u64, Addr> {
    let mut finished = true;
    let mut start_after = None;
    let mut dao_return: HashMap<u64, Addr> = HashMap::new();
    while finished {
        let msg = enterprise_factory_api::msg::QueryMsg::AllDaos(
            enterprise_factory_api::api::QueryAllDaosMsg {
                start_after,
                limit: None,
            },
        );

        let res: AllDaosResponse = app.wrap().query_wasm_smart(ADDR_FACTORY, &msg).unwrap();

        for (i, dao_addr) in res.daos.clone().into_iter().enumerate() {
            dao_return.insert(
                start_after.unwrap_or(Uint64::zero()).u64() + i as u64 + 1,
                dao_addr.dao_address,
            );
        }

        if res.daos.len() == 0 {
            finished = false
        } else {
            start_after =
                Some(start_after.unwrap_or(0_u64.into()) + Uint64::from(res.daos.len() as u64))
        }
    }

    dao_return
}

pub fn qy_get_dao_by_id(app: &App, id: u64) -> Option<Addr> {
    qy_get_all_dao(app).get(&id).map(|addr| addr.clone())
}

pub fn qy_get_all_contracts_of_a_dao(app: &App, id: u64) -> Addresses {
    let treasury_addr = qy_get_dao_by_id(app, id).unwrap();

    let enterprise_addr = enterprise_addr_from_treasury_addr(app, treasury_addr);

    let msg = enterprise_protocol::msg::QueryMsg::ComponentContracts {};

    let res: ComponentContractsResponse = app
        .wrap()
        .query_wasm_smart(enterprise_addr.clone(), &msg)
        .unwrap();

    Addresses {
        enterprise: enterprise_addr,
        others: res,
    }
}

pub fn enterprise_addr_from_treasury_addr(app: &App, treasury_addr: Addr) -> Addr {
    let msg = enterprise_treasury_api::msg::QueryMsg::Config {};

    let res: enterprise_treasury_api::api::ConfigResponse = app
        .wrap()
        .query_wasm_smart(treasury_addr.clone(), &msg)
        .unwrap();
    let governance_controller_addr = res.admin;

    let msg = enterprise_governance_controller_api::msg::QueryMsg::Config {};

    let res: enterprise_governance_controller_api::api::ConfigResponse = app
        .wrap()
        .query_wasm_smart(governance_controller_addr.clone(), &msg)
        .unwrap();

    res.enterprise_contract
}

pub fn qy_cw20_balance(
    app: &App,
    cw20_addr: impl Into<String>,
    address: impl Into<String>,
) -> Uint128 {
    app.wrap()
        .query_wasm_smart::<cw20::BalanceResponse>(
            cw20_addr,
            &cw20::Cw20QueryMsg::Balance {
                address: address.into(),
            },
        )
        .unwrap()
        .balance
}

pub fn qy_get_dao_type(app: &App, enterprise: impl Into<String>) -> DaoType {
    app.wrap()
        .query_wasm_smart::<DaoInfoResponse>(
            enterprise,
            &enterprise_protocol::msg::QueryMsg::DaoInfo {},
        )
        .unwrap()
        .dao_type
}

pub fn qy_get_membership_underline_from_dao_type(
    app: &App,
    dao_type: &DaoType,
    all_contracts: &Addresses,
) -> String {
    match dao_type {
        DaoType::Denom => {
            app.wrap()
                .query_wasm_smart::<DenomConfigResponse>(
                    all_contracts.others.membership_contract.clone(),
                    &denom_staking_api::msg::QueryMsg::DenomConfig {},
                )
                .unwrap()
                .denom
        }
        DaoType::Token => app
            .wrap()
            .query_wasm_smart::<TokenConfigResponse>(
                all_contracts.others.membership_contract.clone(),
                &token_staking_api::msg::QueryMsg::TokenConfig {},
            )
            .unwrap()
            .token_contract
            .to_string(),
        DaoType::Nft => app
            .wrap()
            .query_wasm_smart::<NftConfigResponse>(
                all_contracts.others.membership_contract.clone(),
                &nft_staking_api::msg::QueryMsg::NftConfig {},
            )
            .unwrap()
            .nft_contract
            .to_string(),
        DaoType::Multisig => unimplemented!(),
    }
}

pub fn qy_multisig_user_weight(
    app: &App,
    membership: impl Into<String>,
    user: impl Into<String>,
) -> Uint128 {
    app.wrap()
        .query_wasm_smart::<UserWeightResponse>(
            membership,
            &multisig_membership_api::msg::QueryMsg::UserWeight(UserWeightParams {
                user: user.into(),
            }),
        )
        .unwrap()
        .weight
}

pub fn qy_multisig_total_weight(app: &App, membership: impl Into<String>) -> Uint128 {
    app.wrap()
        .query_wasm_smart::<TotalWeightResponse>(
            membership,
            &multisig_membership_api::msg::QueryMsg::TotalWeight(TotalWeightParams {
                expiration: cw_utils::Expiration::Never {},
            }),
        )
        .unwrap()
        .total_weight
}

pub fn qy_proposal(app: &App, contracts: Addresses, id: u64) -> ProposalResponse {
    app.wrap()
        .query_wasm_smart::<ProposalResponse>(
            contracts.others.enterprise_governance_controller_contract,
            &enterprise_governance_controller_api::msg::QueryMsg::Proposal(ProposalParams {
                proposal_id: id,
            }),
        )
        .unwrap()
}

pub fn qy_membership_user_weight(
    app: &App,
    contracts: &Addresses,
    user: impl Into<String>,
) -> Uint128 {
    app.wrap()
        .query_wasm_smart::<UserWeightResponse>(
            contracts.others.membership_contract.clone(),
            &token_staking_api::msg::QueryMsg::UserWeight(UserWeightParams { user: user.into() }),
        )
        .unwrap()
        .weight
}

pub fn qy_membership_total_weight(
    app: &App,
    contracts: &Addresses,
    expiration: Option<cw_utils::Expiration>,
) -> Uint128 {
    app.wrap()
        .query_wasm_smart::<TotalWeightResponse>(
            contracts.others.membership_contract.clone(),
            &token_staking_api::msg::QueryMsg::TotalWeight(TotalWeightParams {
                expiration: expiration.unwrap_or(cw_utils::Expiration::Never {}),
            }),
        )
        .unwrap()
        .total_weight
}

// pub fn qy_funds_distributor_user_weight(
//     app: &App,
//     contracts: &Addresses,
//     user: impl Into<String>,
// ) -> (Uint128, Uint128) {
//     app.wrap()
//         .query_wasm_smart::<(Uint128, Uint128)>(
//             contracts.others.funds_distributor_contract.clone(),
//             &funds_distributor_api::msg::QueryMsg::UserWeight { user: user.into() },
//         )
//         .unwrap()
// }

// pub fn qy_funds_distributor_total_weight(app: &App, contracts: &Addresses) -> Uint128 {
//     app.wrap()
//         .query_wasm_smart::<Uint128>(
//             contracts.others.funds_distributor_contract.clone(),
//             &funds_distributor_api::msg::QueryMsg::TotalWeight,
//         )
//         .unwrap()
// }

pub fn qy_all_proposals(app: &App, contracts: &Addresses) -> Vec<ProposalResponse> {
    let mut proposals: Vec<ProposalResponse> = vec![];

    let mut msg = ProposalsParams {
        filter: None,
        start_after: None,
        limit: Some(30_u32),
        order: None,
    };

    loop {
        let res = app
            .wrap()
            .query_wasm_smart::<ProposalsResponse>(
                contracts
                    .others
                    .enterprise_governance_controller_contract
                    .clone(),
                &enterprise_governance_controller_api::msg::QueryMsg::Proposals(msg.clone()),
            )
            .unwrap();

        for i in res.proposals.clone() {
            proposals.push(i.clone())
        }

        match res.proposals.last() {
            Some(last) => msg.start_after = Some(last.proposal.id),
            None => break,
        }
    }

    proposals
}

// EXECUTE

pub fn run_membership_deposit(
    app: &mut App,
    sender: impl Into<String> + Clone,
    amount: u128,
    contracts: &Addresses,
) -> AppResult {
    let dao_type = qy_get_dao_type(app, contracts.enterprise.clone());

    let underline = qy_get_membership_underline_from_dao_type(&app, &dao_type, contracts);

    let sender_addr: String = sender.clone().into();

    match dao_type {
        DaoType::Denom => {
            let msg = denom_staking_api::msg::ExecuteMsg::Stake {
                user: Some(sender.into()),
            };
            app.execute_contract(
                sender_addr.into_addr(),
                contracts.others.membership_contract.clone(),
                &msg,
                &[Coin::new(amount, underline)],
            )
        }
        DaoType::Token => {
            let msg = cw20::Cw20ExecuteMsg::Send {
                contract: contracts.others.membership_contract.to_string(),
                amount: amount.into_uint128(),
                msg: to_json_binary(&token_staking_api::msg::Cw20HookMsg::Stake {
                    user: sender.clone().into(),
                })
                .unwrap(),
            };

            app.execute_contract(sender_addr.into_addr(), underline.into_addr(), &msg, &[])
        }
        DaoType::Nft => todo!(),
        DaoType::Multisig => todo!(),
    }
}

pub fn run_membership_unstake_cw20(
    app: &mut App,
    sender: impl Into<String> + Clone,
    amount: u128,
    contracts: &Addresses,
) -> AppResult {
    let msg = token_staking_api::msg::ExecuteMsg::Unstake(UnstakeMsg {
        amount: amount.into(),
    });

    let sender_addr: String = sender.into();

    app.execute_contract(
        sender_addr.into_addr(),
        contracts.others.membership_contract.clone(),
        &msg,
        &[],
    )
}

pub fn run_membership_claim_cw20(
    app: &mut App,
    sender: impl Into<String> + Clone,
    contracts: &Addresses,
) -> AppResult {
    let msg = token_staking_api::msg::ExecuteMsg::Claim(ClaimMsg {
        user: Some(sender.clone().into()),
    });

    let sender_addr: String = sender.into();

    app.execute_contract(
        sender_addr.into_addr(),
        contracts.others.membership_contract.clone(),
        &msg,
        &[],
    )
}

pub fn run_create_council_proposal(
    app: &mut App,
    contracts: Addresses,
    sender: impl Into<String>,
    title: &str,
    description: Option<&str>,
    actions: Vec<ProposalAction>,
) -> AppResult {
    let msg = enterprise_governance_controller_api::msg::ExecuteMsg::CreateCouncilProposal(
        CreateProposalMsg {
            title: title.to_string(),
            description: description.map(|v| v.to_string()),
            proposal_actions: actions,
            deposit_owner: None,
        },
    );

    app.execute_contract(
        sender.into().into_addr(),
        contracts.others.enterprise_governance_controller_contract,
        &msg,
        &[],
    )
}

pub fn run_vote_council_proposal(
    app: &mut App,
    contracts: Addresses,
    sender: impl Into<String>,
    id: u64,
    vote: VoteOutcome,
) -> AppResult {
    let msg = enterprise_governance_controller_api::msg::ExecuteMsg::CastCouncilVote(CastVoteMsg {
        proposal_id: id,
        outcome: vote,
    });

    app.execute_contract(
        sender.into().into_addr(),
        contracts.others.enterprise_governance_controller_contract,
        &msg,
        &[],
    )
}

pub fn run_execute_proposal(
    app: &mut App,
    contracts: Addresses,
    sender: impl Into<String>,
    id: u64,
) -> AppResult {
    let msg = enterprise_governance_controller_api::msg::ExecuteMsg::ExecuteProposal(
        ExecuteProposalMsg { proposal_id: id },
    );

    app.execute_contract(
        sender.into().into_addr(),
        contracts.others.enterprise_governance_controller_contract,
        &msg,
        &[],
    )
}

pub fn run_create_gov_proposal(
    app: &mut App,
    contracts: &Addresses,
    sender: impl Into<String>,
    title: &str,
    description: Option<&str>,
    actions: Vec<ProposalAction>,
    funds: Option<AssetBase<String>>,
) -> AppResult {
    let funds = funds.unwrap_or(AssetBase {
        info: AssetInfoBase::Native("".to_string()),
        amount: 0_u128.into_uint128(),
    });

    let msg =
        enterprise_governance_controller_api::msg::ExecuteMsg::CreateProposal(CreateProposalMsg {
            title: title.to_string(),
            description: description.map(|v| v.to_string()),
            proposal_actions: actions,
            deposit_owner: None,
        });

    match funds.info {
        AssetInfoBase::Native(denom) => {
            let funds: Vec<Coin> = if funds.amount == Uint128::zero() {
                vec![]
            } else {
                vec![Coin::new(funds.amount.u128(), denom)]
            };

            app.execute_contract(
                sender.into().into_addr(),
                contracts
                    .others
                    .enterprise_governance_controller_contract
                    .clone(),
                &msg,
                funds.as_slice(),
            )
        }
        AssetInfoBase::Cw20(contract) => {
            let msg = cw20::Cw20ExecuteMsg::Send {
                contract: contracts
                    .others
                    .enterprise_governance_controller_contract
                    .clone()
                    .to_string(),
                amount: funds.amount,
                msg: to_json_binary(&msg)?,
            };

            app.execute_contract(sender.into().into_addr(), contract.into_addr(), &msg, &[])
        }
        AssetInfoBase::Cw1155(_, _) => todo!(),
        _ => todo!(),
    }
}

pub fn run_vote_gov_proposal(
    app: &mut App,
    contracts: &Addresses,
    sender: impl Into<String>,
    id: u64,
    vote: VoteOutcome,
) -> AppResult {
    let msg = enterprise_governance_controller_api::msg::ExecuteMsg::CastVote(CastVoteMsg {
        proposal_id: id,
        outcome: vote,
    });

    app.execute_contract(
        sender.into().into_addr(),
        contracts
            .others
            .enterprise_governance_controller_contract
            .clone(),
        &msg,
        &[],
    )
}
