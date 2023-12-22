use crate::state::ENTERPRISE_CONTRACT;
use crate::user_weights::update_user_weights_checked;
use cosmwasm_std::DepsMut;
use enterprise_protocol::api::DaoType::Multisig;
use enterprise_protocol::api::{ComponentContractsResponse, DaoInfoResponse};
use enterprise_protocol::msg::QueryMsg::{ComponentContracts, DaoInfo};
use funds_distributor_api::api::{UpdateUserWeightsMsg, UserWeight};
use funds_distributor_api::error::DistributorResult;
use membership_common_api::api::{MembersParams, MembersResponse};
use membership_common_api::msg::QueryMsg::Members;

pub fn migrate_to_v1_0_4(mut deps: DepsMut) -> DistributorResult<()> {
    // there was an issue where multisig DAOs were
    // being created with no weights set to funds distributor
    // we need to now go and re-set weights for each of the members correctly

    let enterprise_contract = ENTERPRISE_CONTRACT.load(deps.storage)?;

    let dao_info: DaoInfoResponse = deps
        .querier
        .query_wasm_smart(enterprise_contract.to_string(), &DaoInfo {})?;

    if dao_info.dao_type == Multisig {
        let component_contracts: ComponentContractsResponse = deps
            .querier
            .query_wasm_smart(enterprise_contract.to_string(), &ComponentContracts {})?;

        let mut last_member: Option<String> = None;
        loop {
            let members: MembersResponse = deps.querier.query_wasm_smart(
                component_contracts.membership_contract.to_string(),
                &Members(MembersParams {
                    start_after: last_member.clone(),
                    limit: None,
                }),
            )?;

            if members.members.is_empty() {
                break;
            }

            update_user_weights_checked(
                deps.branch(),
                UpdateUserWeightsMsg {
                    new_user_weights: members
                        .members
                        .iter()
                        .map(|member| UserWeight {
                            user: member.user.to_string(),
                            weight: member.weight,
                        })
                        .collect(),
                },
            )?;

            last_member = members.members.last().map(|it| it.user.to_string());
        }
    }

    Ok(())
}
