use crate::tests::querier::mock_querier::mock_dependencies;
use crate::validate::validate_dao_council;
use enterprise_protocol::api::DaoCouncilSpec;
use enterprise_protocol::api::ProposalActionType::{
    ExecuteMsgs, ModifyMultisigMembership, RequestFundingFromDao, UpdateCouncil, UpdateGovConfig,
    UpgradeDao,
};
use enterprise_protocol::error::DaoError::{
    DuplicateCouncilMember, UnsupportedCouncilProposalAction,
};

#[test]
fn dao_council_with_duplicate_members_is_invalid() {
    let deps = mock_dependencies();
    let result = validate_dao_council(
        deps.as_ref(),
        Some(DaoCouncilSpec {
            members: vec![
                "member1".to_string(),
                "member2".to_string(),
                "member1".to_string(),
            ],
            allowed_proposal_action_types: Some(vec![UpgradeDao]),
        }),
    );

    assert_eq!(
        result,
        Err(DuplicateCouncilMember {
            member: "member1".to_string()
        })
    );
}

#[test]
fn dao_council_with_invalid_proposal_action_type_is_invalid() {
    let deps = mock_dependencies();
    let invalid_types = vec![
        UpdateGovConfig,
        UpdateCouncil,
        RequestFundingFromDao,
        ExecuteMsgs,
        ModifyMultisigMembership,
    ];

    for invalid_council_action_type in invalid_types {
        let result = validate_dao_council(
            deps.as_ref(),
            Some(DaoCouncilSpec {
                members: vec!["member".to_string()],
                allowed_proposal_action_types: Some(vec![invalid_council_action_type.clone()]),
            }),
        );

        assert_eq!(
            result,
            Err(UnsupportedCouncilProposalAction {
                action: invalid_council_action_type,
            })
        );
    }
}
