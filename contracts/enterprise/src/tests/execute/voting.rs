use crate::tests::helpers::{
    create_council_proposal, existing_token_dao_membership, instantiate_stub_dao, stake_tokens,
    stub_token_info, vote_on_proposal, CW20_ADDR,
};
use crate::tests::querier::mock_querier::mock_dependencies;
use common::cw::testing::{mock_env, mock_info};
use cosmwasm_std::Decimal;
use enterprise_protocol::api::DaoCouncilSpec;
use enterprise_protocol::error::DaoError::Unauthorized;
use enterprise_protocol::error::DaoResult;
use poll_engine_api::api::VoteOutcome::Yes;

#[test]
fn vote_on_council_proposal_by_non_council_member_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
        Some(DaoCouncilSpec {
            members: vec!["council_member".to_string()],
            quorum: Decimal::percent(75),
            threshold: Decimal::percent(50),
            allowed_proposal_action_types: None,
        }),
    )?;

    stake_tokens(deps.as_mut(), &env, CW20_ADDR, "user", 123u128)?;

    create_council_proposal(
        deps.as_mut(),
        &env,
        &mock_info("council_member", &vec![]),
        None,
        None,
        vec![],
    )?;

    let result = vote_on_proposal(deps.as_mut(), &env, "non_council_member", 1, Yes);

    assert_eq!(result, Err(Unauthorized));

    Ok(())
}
