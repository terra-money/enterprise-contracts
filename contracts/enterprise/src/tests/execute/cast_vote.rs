use crate::contract::execute;
use crate::tests::helpers::{
    create_stub_proposal, existing_nft_dao_membership, existing_token_dao_membership,
    instantiate_stub_dao, multisig_dao_membership_info_with_members, stub_token_info, CW20_ADDR,
    NFT_ADDR,
};
use crate::tests::querier::mock_querier::mock_dependencies;
use common::cw::testing::{mock_env, mock_info};
use cosmwasm_std::Uint128;
use enterprise_protocol::api::CastVoteMsg;
use enterprise_protocol::error::DaoError::Unauthorized;
use enterprise_protocol::error::DaoResult;
use enterprise_protocol::msg::ExecuteMsg::CastVote;
use poll_engine_api::api::VoteOutcome::Yes;

#[test]
fn cast_vote_by_non_token_holder_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);
    deps.querier
        .with_token_balances(&[(CW20_ADDR, &[("holder", Uint128::one())])]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
        None,
    )?;

    create_stub_proposal(deps.as_mut(), &env, &mock_info("holder", &vec![]))?;

    let result = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("non_holder", &vec![]),
        CastVote(CastVoteMsg {
            proposal_id: 1,
            outcome: Yes,
        }),
    );

    assert_eq!(result, Err(Unauthorized));

    Ok(())
}

#[test]
fn cast_vote_by_non_nft_holder_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier.with_num_tokens(&[(NFT_ADDR, 1000u64)]);
    deps.querier
        .with_nft_holders(&[(NFT_ADDR, &[("holder", &["1"])])]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        existing_nft_dao_membership(NFT_ADDR),
        None,
        None,
    )?;

    create_stub_proposal(deps.as_mut(), &env, &mock_info("holder", &vec![]))?;

    let result = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("non_holder", &vec![]),
        CastVote(CastVoteMsg {
            proposal_id: 1,
            outcome: Yes,
        }),
    );

    assert_eq!(result, Err(Unauthorized));

    Ok(())
}

#[test]
fn cast_vote_by_non_multisig_member_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    instantiate_stub_dao(
        &mut deps.as_mut(),
        &env,
        &info,
        multisig_dao_membership_info_with_members(&[("member1", 1u64)]),
        None,
        None,
    )?;

    create_stub_proposal(deps.as_mut(), &env, &mock_info("member1", &vec![]))?;

    let result = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("non_member", &vec![]),
        CastVote(CastVoteMsg {
            proposal_id: 1,
            outcome: Yes,
        }),
    );

    assert_eq!(result, Err(Unauthorized));

    Ok(())
}
