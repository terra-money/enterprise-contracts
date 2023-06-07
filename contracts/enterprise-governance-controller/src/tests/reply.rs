use crate::contract::{instantiate, reply};
use crate::tests::helpers::{
    existing_token_dao_membership, stub_dao_gov_config, stub_dao_metadata,
    stub_enterprise_factory_contract, stub_token_info, ENTERPRISE_GOVERNANCE_CODE_ID,
    FUNDS_DISTRIBUTOR_CODE_ID,
};
use crate::tests::querier::mock_querier::mock_dependencies;
use common::cw::testing::{mock_env, mock_info};
use cosmwasm_std::{Reply, SubMsgResponse, SubMsgResult};
use enterprise_protocol::error::DaoResult;
use enterprise_protocol::msg::InstantiateMsg;

#[test]
fn reply_with_unknown_reply_id_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[("cw20_addr", &stub_token_info())]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            enterprise_governance_code_id: ENTERPRISE_GOVERNANCE_CODE_ID,
            funds_distributor_code_id: FUNDS_DISTRIBUTOR_CODE_ID,
            dao_metadata: stub_dao_metadata(),
            dao_gov_config: stub_dao_gov_config(),
            dao_council: None,
            dao_membership_info: existing_token_dao_membership("cw20_addr"),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: None,
            nft_whitelist: None,
            minimum_weight_for_rewards: None,
        },
    )?;

    let result = reply(
        deps.as_mut(),
        env,
        Reply {
            id: 215,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: None,
            }),
        },
    );

    assert!(result.is_err());

    Ok(())
}
