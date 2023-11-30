use crate::contract::{
    COUNCIL_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
    ENTERPRISE_GOVERNANCE_CONTROLLER_INSTANTIATE_REPLY_ID, ENTERPRISE_INSTANTIATE_REPLY_ID,
    ENTERPRISE_OUTPOSTS_INSTANTIATE_REPLY_ID, MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
};
use crate::migration_copy_storage::MIGRATED_USER_WEIGHTS;
use crate::migration_stages::MigrationStage::MigrationCompleted;
use crate::migration_stages::{MigrationStage, MIGRATION_TO_V_1_0_0_STAGE};
use crate::nft_staking::{load_all_nft_stakes_for_user, NFT_STAKES};
use crate::staking::{get_multisig_seconds_checkpoints, get_seconds_checkpoints, CW20_STAKES};
use crate::state::{Config, CONFIG};
use common::commons::ModifyValue;
use common::cw;
use common::cw::{Context, Pagination, ReleaseAt};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::WasmMsg::{Instantiate, Migrate, UpdateAdmin};
use cosmwasm_std::{
    from_json, to_json_binary, wasm_execute, Addr, Binary, BlockInfo, Decimal, Deps, DepsMut, Env,
    Response, StdError, StdResult, Storage, SubMsg, Timestamp, Uint128, Uint64,
};
use cw_asset::{Asset, AssetInfo, AssetInfoUnchecked, AssetUnchecked};
use cw_storage_plus::{Item, Map};
use cw_utils::Duration;
use enterprise_factory_api::api::ConfigResponse;
use enterprise_governance_api::msg::QueryMsg::Polls;
use enterprise_governance_controller_api::api::{
    DaoCouncilSpec, GovConfig, ProposalAction, ProposalActionType, ProposalDeposit,
    ProposalDepositAsset, ProposalId, ProposalInfo, ProposalType,
    UpdateAssetWhitelistProposalActionMsg, UpdateNftWhitelistProposalActionMsg,
};
use enterprise_protocol::api::{
    DaoMetadata, DaoType, FinalizeInstantiationMsg, Logo, VersionMigrateMsg,
};
use enterprise_protocol::msg::ExecuteMsg::FinalizeInstantiation;
use enterprise_treasury_api::error::EnterpriseTreasuryError::{
    InvalidMigrationStage, Std, Unauthorized,
};
use enterprise_treasury_api::error::EnterpriseTreasuryResult;
use enterprise_treasury_api::msg::ExecuteMsg::FinalizeInitialMigrationStep;
use enterprise_versioning_api::api::{Version, VersionInfo, VersionParams, VersionResponse};
use multisig_membership_api::api::UserWeight;
use nft_staking_api::api::NftTokenId;
use nft_staking_api::msg::Cw721HookMsg::AddClaim;
use poll_engine_api::api::{PollsParams, PollsResponse};
use token_staking_api::api::{UserClaim, UserStake};
use token_staking_api::msg::Cw20HookMsg::AddClaims;
use MigrationStage::{MigrationInProgress, MigrationNotStarted};

const DEFAULT_CW20_SUBMSGS_LIMIT: u32 = 100;
const DEFAULT_NFT_SUBMSGS_LIMIT: u32 = 20;
const INITIAL_NFT_MIGRATION_STEP_SUBMSGS_LIMIT: u32 = 5;
const INITIAL_CW20_MIGRATION_STEP_SUBMSGS_LIMIT: u32 = 20;

const POLLS_QUERY_LIMIT: u64 = 30;

const DAO_GOV_CONFIG: Item<DaoGovConfig> = Item::new("dao_gov_config");
const DAO_COUNCIL: Item<Option<DaoCouncil>> = Item::new("dao_council");

const DAO_METADATA: Item<DaoMetadata> = Item::new("dao_metadata");
const DAO_TYPE: Item<DaoType> = Item::new("dao_type");

const DAO_CREATION_DATE: Item<Timestamp> = Item::new("dao_creation_date");

const DAO_MEMBERSHIP_CONTRACT: Item<Addr> = Item::new("dao_membership_contract");

const DAO_CODE_VERSION: Item<Uint64> = Item::new("dao_code_version");

const MULTISIG_MEMBERS: Map<Addr, Uint128> = Map::new("multisig_members");

const ENTERPRISE_GOVERNANCE_CONTRACT: Item<Addr> = Item::new("enterprise_governance_contract");
const FUNDS_DISTRIBUTOR_CONTRACT: Item<Addr> = Item::new("funds_distributor_contract");

const ENTERPRISE_FACTORY_CONTRACT: Item<Addr> = Item::new("enterprise_factory_contract");

const PROPOSAL_INFOS: Map<ProposalId, ProposalInfoV5> = Map::new("proposal_infos");

pub const CLAIMS: Map<&Addr, Vec<Claim>> = Map::new("claims");

#[cw_serde]
pub struct Claim {
    pub asset: ClaimAsset,
    pub release_at: ReleaseAt,
}

#[cw_serde]
pub enum ClaimAsset {
    Cw20(Cw20ClaimAsset),
    Cw721(Cw721ClaimAsset),
}

#[cw_serde]
pub struct Cw20ClaimAsset {
    pub amount: Uint128,
}

#[cw_serde]
pub struct Cw721ClaimAsset {
    pub tokens: Vec<NftTokenId>,
}

#[cw_serde]
pub struct ProposalInfoV5 {
    pub proposal_type: ProposalType,
    pub executed_at: Option<BlockInfo>,
    pub proposal_deposit: Option<ProposalDepositV5>,
    pub proposal_actions: Vec<ProposalActionV5>,
}

impl ProposalInfoV5 {
    pub fn to_proposal_info(self, token_addr: Addr) -> ProposalInfo {
        ProposalInfo {
            proposal_type: self.proposal_type,
            executed_at: self.executed_at,
            earliest_execution: None,
            proposal_deposit: self.proposal_deposit.map(|deposit| ProposalDeposit {
                depositor: deposit.depositor,
                asset: ProposalDepositAsset::Cw20 {
                    token_addr,
                    amount: deposit.amount,
                },
            }),
            proposal_actions: self
                .proposal_actions
                .into_iter()
                .map(|it| it.to_proposal_action())
                .collect(),
        }
    }
}

impl ProposalActionV5 {
    pub fn to_proposal_action(self) -> ProposalAction {
        match self {
            ProposalActionV5::UpdateMetadata(msg) => {
                ProposalAction::UpdateMetadata(enterprise_protocol::api::UpdateMetadataMsg {
                    name: msg.name,
                    description: msg.description,
                    logo: msg.logo,
                    github_username: msg.github_username,
                    discord_username: msg.discord_username,
                    twitter_username: msg.twitter_username,
                    telegram_username: msg.telegram_username,
                })
            }
            ProposalActionV5::UpdateGovConfig(msg) => ProposalAction::UpdateGovConfig(
                enterprise_governance_controller_api::api::UpdateGovConfigMsg {
                    quorum: msg.quorum,
                    threshold: msg.threshold,
                    veto_threshold: msg.veto_threshold,
                    voting_duration: msg.voting_duration,
                    unlocking_period: msg.unlocking_period,
                    minimum_deposit: msg.minimum_deposit,
                    allow_early_proposal_execution: msg.allow_early_proposal_execution,
                },
            ),
            ProposalActionV5::UpdateCouncil(msg) => ProposalAction::UpdateCouncil(
                enterprise_governance_controller_api::api::UpdateCouncilMsg {
                    dao_council: msg.dao_council,
                },
            ),
            ProposalActionV5::UpdateAssetWhitelist(msg) => {
                ProposalAction::UpdateAssetWhitelist(UpdateAssetWhitelistProposalActionMsg {
                    remote_treasury_target: None,
                    add: msg.add.into_iter().map(AssetInfoUnchecked::from).collect(),
                    remove: msg
                        .remove
                        .into_iter()
                        .map(AssetInfoUnchecked::from)
                        .collect(),
                })
            }
            ProposalActionV5::UpdateNftWhitelist(msg) => {
                ProposalAction::UpdateNftWhitelist(UpdateNftWhitelistProposalActionMsg {
                    remote_treasury_target: None,
                    add: msg.add.into_iter().map(|addr| addr.to_string()).collect(),
                    remove: msg
                        .remove
                        .into_iter()
                        .map(|addr| addr.to_string())
                        .collect(),
                })
            }
            ProposalActionV5::RequestFundingFromDao(msg) => ProposalAction::RequestFundingFromDao(
                enterprise_governance_controller_api::api::RequestFundingFromDaoMsg {
                    remote_treasury_target: None,
                    recipient: msg.recipient,
                    assets: msg.assets.into_iter().map(AssetUnchecked::from).collect(),
                },
            ),
            ProposalActionV5::UpgradeDao(msg) => {
                ProposalAction::UpgradeDao(enterprise_protocol::api::UpgradeDaoMsg {
                    new_version: Version {
                        major: 0,
                        minor: msg.new_dao_code_id,
                        patch: 0,
                    },
                    migrate_msgs: vec![VersionMigrateMsg {
                        version: Version {
                            major: 0,
                            minor: msg.new_dao_code_id,
                            patch: 0,
                        },
                        migrate_msg: from_json(&msg.migrate_msg).unwrap_or_default(),
                    }],
                })
            }
            ProposalActionV5::ExecuteMsgs(msg) => ProposalAction::ExecuteMsgs(
                enterprise_governance_controller_api::api::ExecuteMsgsMsg {
                    action_type: msg.action_type,
                    msgs: msg.msgs,
                },
            ),
            ProposalActionV5::ModifyMultisigMembership(msg) => {
                ProposalAction::ModifyMultisigMembership(
                    enterprise_governance_controller_api::api::ModifyMultisigMembershipMsg {
                        edit_members: msg
                            .edit_members
                            .into_iter()
                            .map(|it| UserWeight {
                                user: it.address,
                                weight: it.weight,
                            })
                            .collect(),
                    },
                )
            }
            ProposalActionV5::DistributeFunds(msg) => ProposalAction::DistributeFunds(
                enterprise_governance_controller_api::api::DistributeFundsMsg {
                    funds: msg.funds.into_iter().map(AssetUnchecked::from).collect(),
                },
            ),
            ProposalActionV5::UpdateMinimumWeightForRewards(msg) => {
                ProposalAction::UpdateMinimumWeightForRewards(
                    enterprise_governance_controller_api::api::UpdateMinimumWeightForRewardsMsg {
                        minimum_weight_for_rewards: msg.minimum_weight_for_rewards,
                    },
                )
            }
        }
    }
}

#[cw_serde]
pub struct ProposalDepositV5 {
    pub depositor: Addr,
    pub amount: Uint128,
}

#[cw_serde]
pub enum ProposalActionTypeV5 {
    UpdateMetadata,
    UpdateGovConfig,
    UpdateCouncil,
    UpdateAssetWhitelist,
    UpdateNftWhitelist,
    RequestFundingFromDao,
    UpgradeDao,
    ExecuteMsgs,
    ModifyMultisigMembership,
    DistributeFunds,
    UpdateMinimumWeightForRewards,
}

impl ProposalActionTypeV5 {
    pub fn map_to_proposal_action_type(self) -> ProposalActionType {
        match self {
            ProposalActionTypeV5::UpdateMetadata => ProposalActionType::UpdateMetadata,
            ProposalActionTypeV5::UpdateGovConfig => ProposalActionType::UpdateGovConfig,
            ProposalActionTypeV5::UpdateCouncil => ProposalActionType::UpdateCouncil,
            ProposalActionTypeV5::UpdateAssetWhitelist => ProposalActionType::UpdateAssetWhitelist,
            ProposalActionTypeV5::UpdateNftWhitelist => ProposalActionType::UpdateNftWhitelist,
            ProposalActionTypeV5::RequestFundingFromDao => {
                ProposalActionType::RequestFundingFromDao
            }
            ProposalActionTypeV5::UpgradeDao => ProposalActionType::UpgradeDao,
            ProposalActionTypeV5::ExecuteMsgs => ProposalActionType::ExecuteMsgs,
            ProposalActionTypeV5::ModifyMultisigMembership => {
                ProposalActionType::ModifyMultisigMembership
            }
            ProposalActionTypeV5::DistributeFunds => ProposalActionType::DistributeFunds,
            ProposalActionTypeV5::UpdateMinimumWeightForRewards => {
                ProposalActionType::UpdateMinimumWeightForRewards
            }
        }
    }
}

#[cw_serde]
pub enum ProposalActionV5 {
    UpdateMetadata(UpdateMetadataMsg),
    UpdateGovConfig(UpdateGovConfigMsg),
    UpdateCouncil(UpdateCouncilMsg),
    UpdateAssetWhitelist(UpdateAssetWhitelistMsg),
    UpdateNftWhitelist(UpdateNftWhitelistMsg),
    RequestFundingFromDao(RequestFundingFromDaoMsg),
    UpgradeDao(UpgradeDaoMsg),
    ExecuteMsgs(ExecuteMsgsMsg),
    ModifyMultisigMembership(ModifyMultisigMembershipMsg),
    DistributeFunds(DistributeFundsMsg),
    UpdateMinimumWeightForRewards(UpdateMinimumWeightForRewardsMsg),
}

#[cw_serde]
pub struct UpdateMetadataMsg {
    pub name: ModifyValue<String>,
    pub description: ModifyValue<Option<String>>,
    pub logo: ModifyValue<Logo>,
    pub github_username: ModifyValue<Option<String>>,
    pub discord_username: ModifyValue<Option<String>>,
    pub twitter_username: ModifyValue<Option<String>>,
    pub telegram_username: ModifyValue<Option<String>>,
}

#[cw_serde]
pub struct UpdateGovConfigMsg {
    pub quorum: ModifyValue<Decimal>,
    pub threshold: ModifyValue<Decimal>,
    pub veto_threshold: ModifyValue<Option<Decimal>>,
    pub voting_duration: ModifyValue<Uint64>,
    pub unlocking_period: ModifyValue<Duration>,
    pub minimum_deposit: ModifyValue<Option<Uint128>>,
    pub allow_early_proposal_execution: ModifyValue<bool>,
}

#[cw_serde]
pub struct UpdateCouncilMsg {
    pub dao_council: Option<DaoCouncilSpec>,
}

#[cw_serde]
pub struct UpdateAssetWhitelistMsg {
    /// New assets to add to the whitelist. Will ignore assets that are already whitelisted.
    pub add: Vec<AssetInfo>,
    /// Assets to remove from the whitelist. Will ignore assets that are not already whitelisted.
    pub remove: Vec<AssetInfo>,
}

#[cw_serde]
pub struct UpdateNftWhitelistMsg {
    /// New NFTs to add to the whitelist. Will ignore NFTs that are already whitelisted.
    pub add: Vec<Addr>,
    /// NFTs to remove from the whitelist. Will ignore NFTs that are not already whitelisted.
    pub remove: Vec<Addr>,
}

#[cw_serde]
pub struct RequestFundingFromDaoMsg {
    pub recipient: String,
    pub assets: Vec<Asset>,
}

#[cw_serde]
pub struct UpgradeDaoMsg {
    pub new_dao_code_id: u64,
    pub migrate_msg: Binary,
}

#[cw_serde]
pub struct ExecuteMsgsMsg {
    pub action_type: String,
    pub msgs: Vec<String>,
}

#[cw_serde]
pub struct ModifyMultisigMembershipMsg {
    /// Members to be edited.
    /// Can contain existing members, in which case their new weight will be the one specified in
    /// this message. This effectively allows removing of members (by setting their weight to 0).
    pub edit_members: Vec<MultisigMember>,
}

#[cw_serde]
pub struct MultisigMember {
    pub address: String,
    pub weight: Uint128,
}

#[cw_serde]
pub struct DistributeFundsMsg {
    pub funds: Vec<Asset>,
}

#[cw_serde]
pub struct UpdateMinimumWeightForRewardsMsg {
    pub minimum_weight_for_rewards: Uint128,
}

#[cw_serde]
struct MigrationInfo {
    pub version_info: VersionInfo,
    pub enterprise_contract: Option<Addr>,
    pub enterprise_governance_controller_contract: Option<Addr>,
    pub enterprise_outposts_contract: Option<Addr>,
    pub membership_contract: Option<Addr>,
    pub council_membership_contract: Option<Addr>,
    pub initial_submsgs_limit: Option<u32>,
}
const MIGRATION_INFO: Item<MigrationInfo> = Item::new("migration_info");

impl MigrationInfo {
    fn require_enterprise_contract(&self) -> StdResult<Addr> {
        self.enterprise_contract
            .clone()
            .ok_or(StdError::generic_err(
                "invalid state - missing enterprise contract",
            ))
    }
}

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
    pub allowed_proposal_action_types: Vec<ProposalActionTypeV5>,
    pub quorum: Decimal,
    pub threshold: Decimal,
}

pub fn migrate_to_v_1_0_0(
    deps: DepsMut,
    env: Env,
    initial_submsgs_limit: Option<u32>,
) -> EnterpriseTreasuryResult<Vec<SubMsg>> {
    let dao_code_version = DAO_CODE_VERSION.load(deps.storage)?;

    if dao_code_version < Uint64::from(5u8) {
        return Err(StdError::generic_err(format!(
            "code version {} is too old to migrate to 1.0.0",
            dao_code_version
        ))
        .into());
    }

    let enterprise_factory = ENTERPRISE_FACTORY_CONTRACT.load(deps.storage)?;
    let enterprise_factory_config: ConfigResponse = deps.querier.query_wasm_smart(
        enterprise_factory.to_string(),
        &enterprise_factory_api::msg::QueryMsg::Config {},
    )?;

    let enterprise_versioning = enterprise_factory_config.config.enterprise_versioning;

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
            enterprise_contract: None,
            enterprise_governance_controller_contract: None,
            enterprise_outposts_contract: None,
            membership_contract: None,
            council_membership_contract: None,
            initial_submsgs_limit,
        },
    )?;

    let dao_metadata = DAO_METADATA.load(deps.storage)?;

    let enterprise_contract_submsg = create_enterprise_contract(
        deps.as_ref(),
        env.clone(),
        version_info.version.enterprise_code_id,
        enterprise_factory,
        enterprise_versioning,
        dao_metadata,
    )?;

    let finalize_migration_submsg = SubMsg::new(wasm_execute(
        env.contract.address.to_string(),
        &FinalizeInitialMigrationStep {},
        vec![],
    )?);

    Ok(vec![enterprise_contract_submsg, finalize_migration_submsg])
}

pub fn create_enterprise_contract(
    deps: Deps,
    env: Env,
    enterprise_code_id: u64,
    enterprise_factory: Addr,
    enterprise_versioning: Addr,
    dao_metadata: DaoMetadata,
) -> EnterpriseTreasuryResult<SubMsg> {
    let dao_type = DAO_TYPE.load(deps.storage)?;
    let dao_creation_date = DAO_CREATION_DATE.load(deps.storage)?;

    let submsg = SubMsg::reply_on_success(
        Wasm(Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: enterprise_code_id,
            msg: to_json_binary(&enterprise_protocol::msg::InstantiateMsg {
                enterprise_factory_contract: enterprise_factory.to_string(),
                enterprise_versioning_contract: enterprise_versioning.to_string(),
                dao_metadata,
                dao_creation_date: Some(dao_creation_date),
                dao_type,
                dao_version: Version {
                    major: 1,
                    minor: 0,
                    patch: 0,
                },
            })?,
            funds: vec![],
            label: "Enterprise main contract".to_string(),
        }),
        ENTERPRISE_INSTANTIATE_REPLY_ID,
    );

    Ok(submsg)
}

pub fn enterprise_contract_created(
    mut deps: DepsMut,
    enterprise_contract: Addr,
) -> EnterpriseTreasuryResult<Response> {
    let migration_info = MIGRATION_INFO.load(deps.storage)?;

    MIGRATION_INFO.save(
        deps.storage,
        &MigrationInfo {
            enterprise_contract: Some(enterprise_contract.clone()),
            ..migration_info
        },
    )?;

    let governance_controller_submsg =
        create_governance_controller_contract(deps.branch(), enterprise_contract)?;

    Ok(Response::new().add_submessage(governance_controller_submsg))
}

pub fn create_dao_council_membership_contract(
    deps: DepsMut,
    multisig_membership_code_id: u64,
    enterprise_contract: Addr,
    governance_controller_contract: Addr,
) -> EnterpriseTreasuryResult<SubMsg> {
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
            admin: Some(enterprise_contract.to_string()),
            code_id: multisig_membership_code_id,
            msg: to_json_binary(&multisig_membership_api::msg::InstantiateMsg {
                enterprise_contract: enterprise_contract.to_string(),
                initial_weights: Some(council_members),
                weight_change_hooks: Some(vec![governance_controller_contract.to_string()]),
                total_weight_by_height_checkpoints: None,
                total_weight_by_seconds_checkpoints: None,
            })?,
            funds: vec![],
            label: "Dao council membership".to_string(),
        }),
        COUNCIL_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
    );

    Ok(instantiate_dao_council_membership)
}

pub fn council_membership_contract_created(
    deps: DepsMut,
    council_membership_contract: Addr,
) -> EnterpriseTreasuryResult<Response> {
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

pub fn create_governance_controller_contract(
    deps: DepsMut,
    enterprise_contract: Addr,
) -> EnterpriseTreasuryResult<SubMsg> {
    let version_info = MIGRATION_INFO.load(deps.storage)?.version_info;
    let gov_config = DAO_GOV_CONFIG.load(deps.storage)?;
    let dao_council = DAO_COUNCIL.load(deps.storage)?;
    let dao_type = DAO_TYPE.load(deps.storage)?;

    let dao_membership_contract = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

    let proposal_infos = PROPOSAL_INFOS
        .range(deps.storage, None, None, Ascending)
        .map(|res| {
            res.map(|(id, proposal_info_v5)| {
                (
                    id,
                    proposal_info_v5.to_proposal_info(dao_membership_contract.clone()),
                )
            })
        })
        .collect::<StdResult<Vec<(ProposalId, ProposalInfo)>>>()?;

    let submsg = SubMsg::reply_on_success(
        Wasm(Instantiate {
            admin: Some(enterprise_contract.to_string()),
            code_id: version_info.enterprise_governance_controller_code_id,
            msg: to_json_binary(&enterprise_governance_controller_api::msg::InstantiateMsg {
                enterprise_contract: enterprise_contract.to_string(),
                dao_type,
                gov_config: GovConfig {
                    quorum: gov_config.quorum,
                    threshold: gov_config.threshold,
                    veto_threshold: gov_config.veto_threshold,
                    vote_duration: gov_config.vote_duration,
                    minimum_deposit: gov_config.minimum_deposit,
                    allow_early_proposal_execution: gov_config.allow_early_proposal_execution,
                },
                council_gov_config: dao_council.map(|council| DaoCouncilSpec {
                    members: council
                        .members
                        .into_iter()
                        .map(|addr| addr.to_string())
                        .collect(),
                    allowed_proposal_action_types: Some(
                        council
                            .allowed_proposal_action_types
                            .into_iter()
                            .map(|it| it.map_to_proposal_action_type())
                            .collect(),
                    ),
                    quorum: council.quorum,
                    threshold: council.threshold,
                }),
                proposal_infos: Some(proposal_infos),
            })?,
            funds: vec![],
            label: "Enterprise governance controller".to_string(),
        }),
        ENTERPRISE_GOVERNANCE_CONTROLLER_INSTANTIATE_REPLY_ID,
    );

    Ok(submsg)
}

pub fn governance_controller_contract_created(
    mut deps: DepsMut,
    governance_controller_contract: Addr,
) -> EnterpriseTreasuryResult<Response> {
    let migration_info = MIGRATION_INFO.load(deps.storage)?;

    let enterprise_contract = migration_info.require_enterprise_contract()?;
    let version_info: VersionInfo = migration_info.version_info.clone();

    MIGRATION_INFO.save(
        deps.storage,
        &MigrationInfo {
            enterprise_governance_controller_contract: Some(governance_controller_contract.clone()),
            ..migration_info
        },
    )?;

    CONFIG.save(
        deps.storage,
        &Config {
            admin: governance_controller_contract.clone(),
        },
    )?;

    let dao_type = DAO_TYPE.load(deps.storage)?;
    let response = if dao_type == DaoType::Token {
        let deposit_amount: Uint128 = PROPOSAL_INFOS
            .range(deps.storage, None, None, Ascending)
            .collect::<StdResult<Vec<(ProposalId, ProposalInfoV5)>>>()?
            .into_iter()
            .filter_map(|(_, proposal_info)| {
                let was_executed = proposal_info.executed_at.is_some();

                if was_executed {
                    // if proposal was executed, deposit was already either returned or kept
                    None
                } else {
                    // if the proposal wasn't executed, move the deposit
                    proposal_info.proposal_deposit
                }
            })
            .map(|deposit| deposit.amount)
            .sum();

        let response = Response::new();

        if deposit_amount.is_zero() {
            // deposit is zero, nothing to send
            response
        } else {
            // deposit is not zero, send it over to governance controller
            let dao_token = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

            let send_deposits_submsg = Asset::cw20(dao_token, deposit_amount)
                .transfer_msg(governance_controller_contract.to_string())?;

            response.add_submessage(SubMsg::new(send_deposits_submsg))
        }
    } else {
        Response::new()
    };

    let dao_council_membership_submsg = create_dao_council_membership_contract(
        deps.branch(),
        version_info.multisig_membership_code_id,
        enterprise_contract.clone(),
        governance_controller_contract.clone(),
    )?;

    let membership_submsg = create_enterprise_membership_contract(
        deps.branch(),
        version_info.token_staking_membership_code_id,
        version_info.nft_staking_membership_code_id,
        version_info.multisig_membership_code_id,
        enterprise_contract.clone(),
        governance_controller_contract,
    )?;

    let enterprise_outposts_submsg = create_enterprise_outposts_contract(
        enterprise_contract,
        version_info.enterprise_outposts_code_id,
    )?;

    Ok(response
        .add_submessage(dao_council_membership_submsg)
        .add_submessage(membership_submsg)
        .add_submessage(enterprise_outposts_submsg))
}

pub fn create_enterprise_membership_contract(
    deps: DepsMut,
    token_membership_code_id: u64,
    nft_membership_code_id: u64,
    multisig_membership_code_id: u64,
    enterprise_contract: Addr,
    governance_controller_contract: Addr,
) -> EnterpriseTreasuryResult<SubMsg> {
    let gov_config = DAO_GOV_CONFIG.load(deps.storage)?;

    let dao_type = DAO_TYPE.load(deps.storage)?;

    let weight_change_hooks = Some(vec![governance_controller_contract.to_string()]);

    let msg = match dao_type {
        DaoType::Denom => {
            return Err(Std(StdError::generic_err(
                "Denom membership was not supported prior to this migration!",
            )))
        }
        DaoType::Token => {
            let cw20_contract = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

            Wasm(Instantiate {
                admin: Some(enterprise_contract.to_string()),
                code_id: token_membership_code_id,
                msg: to_json_binary(&token_staking_api::msg::InstantiateMsg {
                    enterprise_contract: enterprise_contract.to_string(),
                    token_contract: cw20_contract.to_string(),
                    unlocking_period: gov_config.unlocking_period,
                    weight_change_hooks,
                    total_weight_by_height_checkpoints: None,
                    total_weight_by_seconds_checkpoints: Some(get_seconds_checkpoints(
                        deps.as_ref(),
                        get_proposal_end_times(deps.as_ref())?,
                    )?),
                })?,
                funds: vec![],
                label: "Token staking membership".to_string(),
            })
        }
        DaoType::Nft => {
            let cw721_contract = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

            Wasm(Instantiate {
                admin: Some(enterprise_contract.to_string()),
                code_id: nft_membership_code_id,
                msg: to_json_binary(&nft_staking_api::msg::InstantiateMsg {
                    enterprise_contract: enterprise_contract.to_string(),
                    nft_contract: cw721_contract.to_string(),
                    unlocking_period: gov_config.unlocking_period,
                    weight_change_hooks,
                    total_weight_by_height_checkpoints: None,
                    total_weight_by_seconds_checkpoints: Some(get_seconds_checkpoints(
                        deps.as_ref(),
                        get_proposal_end_times(deps.as_ref())?,
                    )?),
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
                admin: Some(enterprise_contract.to_string()),
                code_id: multisig_membership_code_id,
                msg: to_json_binary(&multisig_membership_api::msg::InstantiateMsg {
                    enterprise_contract: enterprise_contract.to_string(),
                    initial_weights: Some(initial_weights),
                    weight_change_hooks,
                    total_weight_by_height_checkpoints: None,
                    total_weight_by_seconds_checkpoints: Some(get_multisig_seconds_checkpoints(
                        deps.as_ref(),
                        get_proposal_end_times(deps.as_ref())?,
                    )?),
                })?,
                funds: vec![],
                label: "Multisig membership".to_string(),
            })
        }
    };

    let submsg = SubMsg::reply_on_success(msg, MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID);

    Ok(submsg)
}

fn get_proposal_end_times(deps: Deps) -> EnterpriseTreasuryResult<Vec<Timestamp>> {
    let governance_contract = ENTERPRISE_GOVERNANCE_CONTRACT.load(deps.storage)?;

    let mut poll_end_timestamps = vec![];

    let mut start_after: Option<u64> = None;
    loop {
        let polls_response = query_polls(
            deps,
            governance_contract.to_string(),
            start_after.map(Uint64::from),
        )?;

        if polls_response.polls.is_empty() {
            break;
        } else {
            for poll in polls_response.polls {
                start_after = Some(poll.id);
                poll_end_timestamps.push(poll.ends_at);
            }
        }
    }

    Ok(poll_end_timestamps)
}

fn query_polls(
    deps: Deps,
    governance_contract: String,
    start_after: Option<Uint64>,
) -> EnterpriseTreasuryResult<PollsResponse> {
    let polls_response = deps.querier.query_wasm_smart(
        governance_contract,
        &Polls(PollsParams {
            filter: None,
            pagination: Pagination {
                start_after,
                end_at: None,
                limit: Some(POLLS_QUERY_LIMIT),
                order_by: Some(cw::Order::Ascending),
            },
        }),
    )?;

    Ok(polls_response)
}

pub fn membership_contract_created(
    deps: DepsMut,
    membership_contract: Addr,
) -> EnterpriseTreasuryResult<Response> {
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

/// Carries result of an operation that consumes a certain amount of items to perform itself.
// TODO: this is a really shitty name, think of a better one
struct ResultWithItemsConsumed<T> {
    pub result: T,
    pub items_consumed: u32,
}

fn finalize_membership_contract_submsgs(
    mut deps: DepsMut,
    membership_contract: Addr,
    limit: u32,
) -> EnterpriseTreasuryResult<ResultWithItemsConsumed<Vec<SubMsg>>> {
    let dao_type = DAO_TYPE.load(deps.storage)?;

    let finalize_membership_msgs = match dao_type {
        DaoType::Denom => {
            return Err(Std(StdError::generic_err(
                "Denom membership was not supported prior to this migration!",
            )))
        }
        DaoType::Token => {
            let cw20_contract = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

            let ResultWithItemsConsumed {
                result: stakes_submsg,
                items_consumed: items_consumed_stakes,
            } = migrate_and_clear_cw20_stakes_submsg(
                deps.branch(),
                cw20_contract.clone(),
                membership_contract.clone(),
                limit,
            )?;

            let ResultWithItemsConsumed {
                result: claims_submsg,
                items_consumed: items_consumed_claims,
            } = migrate_and_clear_cw20_claims_submsg(
                deps.branch(),
                cw20_contract,
                membership_contract,
                limit - items_consumed_stakes,
            )?;

            let items_consumed = items_consumed_stakes + items_consumed_claims;

            ResultWithItemsConsumed {
                result: vec![stakes_submsg, claims_submsg]
                    .into_iter()
                    .flatten()
                    .collect(),
                items_consumed,
            }
        }
        DaoType::Nft => {
            let cw721_contract = DAO_MEMBERSHIP_CONTRACT.load(deps.storage)?;

            let mut migrate_stakes_submsgs = migrate_and_clear_cw721_stakes_submsgs(
                deps.branch(),
                cw721_contract.clone(),
                membership_contract.clone(),
                limit,
            )?;

            let remaining_limit = limit - migrate_stakes_submsgs.len() as u32;

            let mut claim_submsgs = migrate_and_clear_cw721_claims_submsgs(
                deps,
                cw721_contract,
                membership_contract,
                remaining_limit,
            )?;

            migrate_stakes_submsgs.append(&mut claim_submsgs);

            let items_consumed = migrate_stakes_submsgs.len() as u32;

            ResultWithItemsConsumed {
                result: migrate_stakes_submsgs,
                items_consumed,
            }
        }
        DaoType::Multisig => {
            // nothing to finalize in multisig DAOs
            ResultWithItemsConsumed {
                result: vec![],
                items_consumed: 0,
            } // TODO: will this prevent multisigs from being finalized? test
        }
    };

    Ok(finalize_membership_msgs)
}

fn migrate_and_clear_cw20_stakes_submsg(
    deps: DepsMut,
    cw20_contract: Addr,
    membership_contract: Addr,
    limit: u32,
) -> EnterpriseTreasuryResult<ResultWithItemsConsumed<Option<SubMsg>>> {
    let mut total_stakes = Uint128::zero();
    let mut stakers_to_remove: Vec<(Addr, Uint128)> = vec![];

    let stakers = CW20_STAKES
        .range(deps.storage, None, None, Ascending)
        .take(limit as usize)
        .map(|res| {
            res.map(|(user, amount)| {
                total_stakes += amount;
                stakers_to_remove.push((user.clone(), amount));
                UserStake {
                    user: user.to_string(),
                    staked_amount: amount,
                }
            })
        })
        .collect::<StdResult<Vec<UserStake>>>()?;

    let items_consumed = stakers_to_remove.len() as u32;

    for (staker, weight) in stakers_to_remove {
        CW20_STAKES.remove(deps.storage, staker.clone());
        MIGRATED_USER_WEIGHTS.save(deps.storage, staker, &weight)?;
    }

    if total_stakes.is_zero() {
        Ok(ResultWithItemsConsumed {
            result: None,
            items_consumed,
        })
    } else {
        Ok(ResultWithItemsConsumed {
            result: Some(SubMsg::new(
                Asset::cw20(cw20_contract, total_stakes).send_msg(
                    membership_contract,
                    to_json_binary(&token_staking_api::msg::Cw20HookMsg::AddStakes { stakers })?,
                )?,
            )),
            items_consumed,
        })
    }
}

fn migrate_and_clear_cw20_claims_submsg(
    deps: DepsMut,
    cw20_contract: Addr,
    membership_contract: Addr,
    limit: u32,
) -> EnterpriseTreasuryResult<ResultWithItemsConsumed<Option<SubMsg>>> {
    let mut total_claims_amount = Uint128::zero();

    let mut claims_included = 0u32;

    let mut claims_to_send: Vec<UserClaim> = vec![];

    while claims_included < limit && !CLAIMS.is_empty(deps.storage) {
        let mut claims_to_replace: Vec<(Addr, Vec<Claim>)> = vec![];

        for claim in CLAIMS
            .range(deps.storage, None, None, Ascending)
            .take(limit as usize)
        {
            if claims_included >= limit {
                break;
            }

            let (user, claims) = claim?;

            let mut claims_remaining: Vec<Claim> = vec![];

            for claim in claims {
                match claim.asset {
                    ClaimAsset::Cw20(asset) if claims_included < limit => {
                        total_claims_amount += asset.amount;
                        claims_included += 1;
                        claims_to_send.push(UserClaim {
                            user: user.to_string(),
                            claim_amount: asset.amount,
                            release_at: claim.release_at,
                        });
                    }
                    _ => claims_remaining.push(claim),
                }
            }

            claims_to_replace.push((user, claims_remaining));
        }

        for (user, claims_remaining) in claims_to_replace {
            if claims_remaining.is_empty() {
                CLAIMS.remove(deps.storage, &user);
            } else {
                CLAIMS.save(deps.storage, &user, &claims_remaining)?;
            }
        }
    }

    let items_consumed = claims_included;

    if total_claims_amount.is_zero() {
        Ok(ResultWithItemsConsumed {
            result: None,
            items_consumed,
        })
    } else {
        let migrate_claims_submsg = SubMsg::new(wasm_execute(
            cw20_contract.to_string(),
            &cw20::Cw20ExecuteMsg::Send {
                contract: membership_contract.to_string(),
                amount: total_claims_amount,
                msg: to_json_binary(&AddClaims {
                    claims: claims_to_send,
                })?,
            },
            vec![],
        )?);

        Ok(ResultWithItemsConsumed {
            result: Some(migrate_claims_submsg),
            items_consumed,
        })
    }
}

fn migrate_and_clear_cw721_stakes_submsgs(
    deps: DepsMut,
    cw721_contract: Addr,
    membership_contract: Addr,
    limit: u32,
) -> EnterpriseTreasuryResult<Vec<SubMsg>> {
    let mut migrate_stakes_submsgs = vec![];

    let mut stakes_to_remove: Vec<(NftTokenId, Addr)> = vec![];

    for stake_res in NFT_STAKES()
        .range(deps.storage, None, None, Ascending)
        .take(limit as usize)
    {
        let (key, stake) = stake_res?;

        let submsg = wasm_execute(
            cw721_contract.to_string(),
            &cw721::Cw721ExecuteMsg::SendNft {
                contract: membership_contract.to_string(),
                token_id: stake.token_id,
                msg: to_json_binary(&nft_staking_api::msg::Cw721HookMsg::Stake {
                    user: stake.staker.to_string(),
                })?,
            },
            vec![],
        )?;

        migrate_stakes_submsgs.push(SubMsg::new(submsg));

        stakes_to_remove.push((key, stake.staker));
    }

    for (stake_key, staker) in stakes_to_remove {
        NFT_STAKES().remove(deps.storage, stake_key)?;

        let previously_migrated_stake = MIGRATED_USER_WEIGHTS
            .may_load(deps.storage, staker.clone())?
            .unwrap_or_default();
        MIGRATED_USER_WEIGHTS.save(
            deps.storage,
            staker,
            &(previously_migrated_stake.checked_add(Uint128::one())?),
        )?;
    }

    Ok(migrate_stakes_submsgs)
}

fn migrate_and_clear_cw721_claims_submsgs(
    deps: DepsMut,
    cw721_contract: Addr,
    membership_contract: Addr,
    limit: u32,
) -> EnterpriseTreasuryResult<Vec<SubMsg>> {
    let mut claim_submsgs = vec![];

    let mut claims_included = 0u32;

    while claims_included < limit && !CLAIMS.is_empty(deps.storage) {
        let mut claim_keys_to_remove: Vec<Addr> = vec![];

        for claim_res in CLAIMS
            .range(deps.storage, None, None, Ascending)
            .take(limit as usize)
        {
            let (user, claims) = claim_res?;

            for claim in claims {
                match claim.asset {
                    ClaimAsset::Cw20(_) => continue,
                    ClaimAsset::Cw721(asset) => {
                        for token in asset.tokens {
                            let submsg = SubMsg::new(wasm_execute(
                                cw721_contract.to_string(),
                                &cw721::Cw721ExecuteMsg::SendNft {
                                    contract: membership_contract.to_string(),
                                    token_id: token,
                                    msg: to_json_binary(&AddClaim {
                                        user: user.to_string(),
                                        release_at: claim.release_at.clone(),
                                    })?,
                                },
                                vec![],
                            )?);
                            claim_submsgs.push(submsg);
                            claims_included += 1;
                        }
                    }
                }
            }

            claim_keys_to_remove.push(user);
        }

        for claim_key in claim_keys_to_remove {
            CLAIMS.remove(deps.storage, &claim_key);
        }
    }

    Ok(claim_submsgs)
}

pub fn create_enterprise_outposts_contract(
    enterprise_contract: Addr,
    enterprise_outposts_code_id: u64,
) -> EnterpriseTreasuryResult<SubMsg> {
    let instantiate_enterprise_outposts = SubMsg::reply_on_success(
        Wasm(Instantiate {
            admin: Some(enterprise_contract.to_string()),
            code_id: enterprise_outposts_code_id,
            msg: to_json_binary(&enterprise_outposts_api::msg::InstantiateMsg {
                enterprise_contract: enterprise_contract.to_string(),
            })?,
            funds: vec![],
            label: "Enterprise outposts".to_string(),
        }),
        ENTERPRISE_OUTPOSTS_INSTANTIATE_REPLY_ID,
    );

    Ok(instantiate_enterprise_outposts)
}

pub fn enterprise_outposts_contract_created(
    deps: DepsMut,
    enterprise_outposts_contract: Addr,
) -> EnterpriseTreasuryResult<Response> {
    let migration_info = MIGRATION_INFO.load(deps.storage)?;

    MIGRATION_INFO.save(
        deps.storage,
        &MigrationInfo {
            enterprise_outposts_contract: Some(enterprise_outposts_contract),
            ..migration_info
        },
    )?;

    Ok(Response::new())
}

pub fn finalize_initial_migration_step(ctx: &mut Context) -> EnterpriseTreasuryResult<Response> {
    if ctx.info.sender != ctx.env.contract.address {
        return Err(Unauthorized);
    }

    let migration_info = MIGRATION_INFO.load(ctx.deps.storage)?;

    let enterprise_contract = migration_info.require_enterprise_contract()?;
    let version_info = migration_info.version_info;

    let governance_controller_contract = migration_info
        .enterprise_governance_controller_contract
        .ok_or(Std(StdError::generic_err(
        "invalid state - missing governance controller address",
    )))?;
    let enterprise_outposts_contract =
        migration_info
            .enterprise_outposts_contract
            .ok_or(Std(StdError::generic_err(
                "invalid state - missing outposts address",
            )))?;
    let membership_contract =
        migration_info
            .membership_contract
            .ok_or(Std(StdError::generic_err(
                "invalid state - missing membership address",
            )))?;
    let council_membership_contract =
        migration_info
            .council_membership_contract
            .ok_or(Std(StdError::generic_err(
                "invalid state - missing council membership address",
            )))?;

    let funds_distributor = FUNDS_DISTRIBUTOR_CONTRACT.load(ctx.deps.storage)?;
    let migrate_funds_distributor = SubMsg::new(Wasm(Migrate {
        contract_addr: funds_distributor.to_string(),
        new_code_id: version_info.funds_distributor_code_id,
        msg: to_json_binary(&funds_distributor_api::msg::MigrateMsg {
            new_admin: governance_controller_contract.to_string(),
            new_enterprise_contract: enterprise_contract.to_string(),
        })?,
    }));

    let enterprise_governance = ENTERPRISE_GOVERNANCE_CONTRACT.load(ctx.deps.storage)?;
    let migrate_enterprise_governance_submsg = SubMsg::new(Wasm(Migrate {
        contract_addr: enterprise_governance.to_string(),
        new_code_id: version_info.enterprise_governance_code_id,
        msg: to_json_binary(&enterprise_governance_api::msg::MigrateMsg {
            new_admin: governance_controller_contract.to_string(),
        })?,
    }));

    // update enterprise's admin to itself
    let update_enterprise_admin = SubMsg::new(UpdateAdmin {
        contract_addr: enterprise_contract.to_string(),
        admin: enterprise_contract.to_string(),
    });

    // update all existing contracts' admin to enterprise
    let update_treasury_admin = SubMsg::new(UpdateAdmin {
        contract_addr: ctx.env.contract.address.to_string(),
        admin: enterprise_contract.to_string(),
    });

    let update_funds_distributor_admin = SubMsg::new(UpdateAdmin {
        contract_addr: funds_distributor.to_string(),
        admin: enterprise_contract.to_string(),
    });

    let update_enterprise_governance_admin = SubMsg::new(UpdateAdmin {
        contract_addr: enterprise_governance.to_string(),
        admin: enterprise_contract.to_string(),
    });

    let finalize_enterprise_instantiation = SubMsg::new(wasm_execute(
        enterprise_contract.to_string(),
        &FinalizeInstantiation(FinalizeInstantiationMsg {
            enterprise_treasury_contract: ctx.env.contract.address.to_string(),
            enterprise_governance_contract: enterprise_governance.to_string(),
            enterprise_governance_controller_contract: governance_controller_contract.to_string(),
            enterprise_outposts_contract: enterprise_outposts_contract.to_string(),
            funds_distributor_contract: funds_distributor.to_string(),
            membership_contract: membership_contract.to_string(),
            council_membership_contract: council_membership_contract.to_string(),
            attestation_contract: None,
        }),
        vec![],
    )?);

    let dao_type = DAO_TYPE.load(ctx.deps.storage)?;

    let initial_submsgs_limit = migration_info
        .initial_submsgs_limit
        .unwrap_or(match dao_type {
            DaoType::Nft => INITIAL_NFT_MIGRATION_STEP_SUBMSGS_LIMIT,
            DaoType::Denom | DaoType::Token | DaoType::Multisig => {
                INITIAL_CW20_MIGRATION_STEP_SUBMSGS_LIMIT
            }
        });

    let ResultWithItemsConsumed {
        result: finalize_membership_contract_submsgs,
        items_consumed,
    } = finalize_membership_contract_submsgs(
        ctx.deps.branch(),
        membership_contract,
        initial_submsgs_limit,
    )?;

    if items_consumed < initial_submsgs_limit {
        // there were fewer items than the maximum we allow, just finalize the whole migration
        set_migration_stage_to_completed(ctx.deps.storage)?;
    } else {
        set_migration_stage_to_in_progress(ctx.deps.storage)?;
    }

    Ok(Response::new()
        .add_submessage(migrate_funds_distributor)
        .add_submessage(migrate_enterprise_governance_submsg)
        .add_submessage(update_enterprise_admin)
        .add_submessage(update_treasury_admin)
        .add_submessage(update_funds_distributor_admin)
        .add_submessage(update_enterprise_governance_admin)
        .add_submessage(finalize_enterprise_instantiation)
        .add_submessages(finalize_membership_contract_submsgs))
}

pub fn perform_next_migration_step(
    ctx: &mut Context,
    submsgs_limit: Option<u32>,
) -> EnterpriseTreasuryResult<Response> {
    let migration_stage = MIGRATION_TO_V_1_0_0_STAGE
        .may_load(ctx.deps.storage)?
        .unwrap_or(MigrationNotStarted);

    match migration_stage {
        MigrationNotStarted | MigrationCompleted => {
            // not allowed to perform the operation
            Err(InvalidMigrationStage)
        }
        MigrationInProgress => perform_next_migration_step_safe(ctx, submsgs_limit),
    }
}

fn perform_next_migration_step_safe(
    ctx: &mut Context,
    submsgs_limit: Option<u32>,
) -> EnterpriseTreasuryResult<Response> {
    let migration_info = MIGRATION_INFO.load(ctx.deps.storage)?;
    let membership_contract =
        migration_info
            .membership_contract
            .ok_or(Std(StdError::generic_err(
                "invalid state - missing membership address",
            )))?;

    let dao_type = DAO_TYPE.load(ctx.deps.storage)?;

    let limit = submsgs_limit.unwrap_or(match dao_type {
        DaoType::Nft => DEFAULT_NFT_SUBMSGS_LIMIT,
        DaoType::Denom | DaoType::Token | DaoType::Multisig => DEFAULT_CW20_SUBMSGS_LIMIT,
    });

    let ResultWithItemsConsumed {
        result: submsgs,
        items_consumed,
    } = finalize_membership_contract_submsgs(ctx.deps.branch(), membership_contract, limit)?;

    if items_consumed < limit {
        set_migration_stage_to_completed(ctx.deps.storage)?;
    } else {
        set_migration_stage_to_in_progress(ctx.deps.storage)?;
    };

    Ok(Response::new()
        .add_attribute("action", "perform_next_migration_step")
        .add_submessages(submsgs))
}

fn set_migration_stage_to_in_progress(storage: &mut dyn Storage) -> EnterpriseTreasuryResult<()> {
    MIGRATION_TO_V_1_0_0_STAGE.save(storage, &MigrationInProgress)?;

    Ok(())
}

fn set_migration_stage_to_completed(storage: &mut dyn Storage) -> EnterpriseTreasuryResult<()> {
    MIGRATION_TO_V_1_0_0_STAGE.save(storage, &MigrationCompleted)?;

    Ok(())
}

/// Loads a user's weight from before migration.
/// As the migration moves weights over to the membership contract, this weight will be removed
/// at some point.
pub fn load_pre_migration_user_weight(
    deps: Deps,
    user: Addr,
) -> EnterpriseTreasuryResult<Option<Uint128>> {
    let dao_type = DAO_TYPE.load(deps.storage)?;

    let weight = match dao_type {
        DaoType::Denom => {
            return Err(StdError::generic_err("No denom DAOs existed pre-migration!").into());
        }
        DaoType::Token => CW20_STAKES.may_load(deps.storage, user)?,
        DaoType::Nft => load_all_nft_stakes_for_user(deps.storage, user)?,
        DaoType::Multisig => MULTISIG_MEMBERS.may_load(deps.storage, user)?,
    };

    Ok(weight)
}
