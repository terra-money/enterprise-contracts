use crate::proposals::{ProposalInfo, PROPOSAL_INFOS};
use crate::state::{DAO_COUNCIL, DAO_METADATA, DAO_METADATA_KEY};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{BlockInfo, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use enterprise_protocol::api::{
    DaoMetadata, DaoSocialData, ExecuteMsgsMsg, Logo, ModifyMultisigMembershipMsg, ProposalAction,
    ProposalDeposit, ProposalId, RequestFundingFromDaoMsg, UpdateAssetWhitelistMsg,
    UpdateCouncilMsg, UpdateGovConfigMsg, UpdateMetadataMsg, UpdateNftWhitelistMsg, UpgradeDaoMsg,
};
use enterprise_protocol::error::DaoResult;
use itertools::Itertools;

pub fn migrate_v1_to_v2(store: &mut dyn Storage) -> DaoResult<()> {
    DAO_COUNCIL.save(store, &None)?;

    migrate_dao_metadata_v1(store)?;
    migrate_proposal_infos_v1(store)?;

    poll_engine::migration::migrate_v1_to_v2(store)?;

    Ok(())
}

#[cw_serde]
struct DaoMetadataV1 {
    pub name: String,
    pub logo: Logo,
    pub socials: DaoSocialData,
}

#[cw_serde]
struct ProposalInfoV1 {
    pub executed_at: Option<BlockInfo>,
    pub proposal_deposit: Option<ProposalDeposit>,
    pub proposal_actions: Vec<ProposalActionV1>,
}

#[cw_serde]
enum ProposalActionV1 {
    UpdateMetadata(UpdateMetadataMsg),
    UpdateGovConfig(UpdateGovConfigMsg),
    UpdateCouncil(UpdateCouncilMsg),
    UpdateAssetWhitelist(UpdateAssetWhitelistMsg),
    UpdateNftWhitelist(UpdateNftWhitelistMsg),
    RequestFundingFromDao(RequestFundingFromDaoMsg),
    UpgradeDao(UpgradeDaoMsg),
    ExecuteMsgs(ExecuteMsgsMsgV1),
    ModifyMultisigMembership(ModifyMultisigMembershipMsg),
}

impl ProposalActionV1 {
    pub fn into_proposal_action(self) -> ProposalAction {
        match self {
            ProposalActionV1::UpdateMetadata(msg) => ProposalAction::UpdateMetadata(msg),
            ProposalActionV1::UpdateGovConfig(msg) => ProposalAction::UpdateGovConfig(msg),
            ProposalActionV1::UpdateCouncil(msg) => ProposalAction::UpdateCouncil(msg),
            ProposalActionV1::UpdateAssetWhitelist(msg) => {
                ProposalAction::UpdateAssetWhitelist(msg)
            }
            ProposalActionV1::UpdateNftWhitelist(msg) => ProposalAction::UpdateNftWhitelist(msg),
            ProposalActionV1::RequestFundingFromDao(msg) => {
                ProposalAction::RequestFundingFromDao(msg)
            }
            ProposalActionV1::UpgradeDao(msg) => ProposalAction::UpgradeDao(msg),
            ProposalActionV1::ExecuteMsgs(msg) => ProposalAction::ExecuteMsgs(ExecuteMsgsMsg {
                action_type: "unknown".to_string(),
                msgs: msg.msgs,
            }),
            ProposalActionV1::ModifyMultisigMembership(msg) => {
                ProposalAction::ModifyMultisigMembership(msg)
            }
        }
    }
}

#[cw_serde]
struct ExecuteMsgsMsgV1 {
    pub msgs: Vec<String>,
}

fn migrate_dao_metadata_v1(store: &mut dyn Storage) -> DaoResult<()> {
    let metadata_v1: DaoMetadataV1 = Item::new(DAO_METADATA_KEY).load(store)?;

    let metadata = DaoMetadata {
        name: metadata_v1.name,
        description: None,
        logo: metadata_v1.logo,
        socials: metadata_v1.socials,
    };
    DAO_METADATA.save(store, &metadata)?;

    Ok(())
}

fn migrate_proposal_infos_v1(store: &mut dyn Storage) -> DaoResult<()> {
    let proposal_infos_v1: Map<ProposalId, ProposalInfoV1> = Map::new("proposal_infos");

    proposal_infos_v1
        .range(store, None, None, Ascending)
        .collect::<StdResult<Vec<(ProposalId, ProposalInfoV1)>>>()?
        .into_iter()
        .map(|(id, info)| {
            (
                id,
                ProposalInfo {
                    executed_at: info.executed_at,
                    proposal_deposit: info.proposal_deposit,
                    proposal_actions: info
                        .proposal_actions
                        .into_iter()
                        .map(|action| action.into_proposal_action())
                        .collect_vec(),
                },
            )
        })
        .try_for_each(|(id, info)| PROPOSAL_INFOS.save(store, id, &info))?;

    Ok(())
}
