use crate::contract::query_list_multisig_members;
use crate::tests::helpers::{
    existing_nft_dao_membership, existing_token_dao_membership, instantiate_stub_dao,
    stub_token_info, CW20_ADDR, NFT_ADDR,
};
use crate::tests::querier::mock_querier::mock_dependencies;
use common::cw::testing::{mock_env, mock_info, mock_query_ctx};
use enterprise_protocol::api::ListMultisigMembersMsg;
use enterprise_protocol::error::DaoError::UnsupportedOperationForDaoType;
use enterprise_protocol::error::DaoResult;

#[test]
fn query_token_dao_multisig_members_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
    )?;

    let result = query_list_multisig_members(
        mock_query_ctx(deps.as_ref(), &env),
        ListMultisigMembersMsg {
            start_after: None,
            limit: None,
        },
    );

    assert_eq!(
        result,
        Err(UnsupportedOperationForDaoType {
            dao_type: "Token".to_string()
        })
    );

    Ok(())
}

#[test]
fn query_nft_dao_multisig_members_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier.with_num_tokens(&[(NFT_ADDR, 100u64)]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_nft_dao_membership(NFT_ADDR),
        None,
    )?;

    let result = query_list_multisig_members(
        mock_query_ctx(deps.as_ref(), &env),
        ListMultisigMembersMsg {
            start_after: None,
            limit: None,
        },
    );

    assert_eq!(
        result,
        Err(UnsupportedOperationForDaoType {
            dao_type: "Nft".to_string()
        })
    );

    Ok(())
}
