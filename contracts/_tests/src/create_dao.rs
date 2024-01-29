use crate::asset_helpers::cw20_unchecked;
use crate::facade_helpers::TestFacade;
use crate::factory_helpers::{
    default_new_token_membership, get_first_dao, new_token_membership, query_all_daos,
};
use crate::helpers::{
    startup_with_versioning, ADDR_FACTORY, CW20_TOKEN1, CW20_TOKEN2, NFT_TOKEN1, NFT_TOKEN2, USER1,
    USER2, USER_DAO_CREATOR,
};
use crate::traits::{IntoAddr, IntoStringVec};
use cosmwasm_std::Decimal;
use cw_multi_test::Executor;
use enterprise_facade_common::facade::EnterpriseFacade;
use enterprise_factory_api::api::{AllDaosResponse, CreateDaoMsg, QueryAllDaosMsg};
use enterprise_factory_api::msg::QueryMsg::AllDaos;
use enterprise_governance_controller_api::api::{DaoCouncilSpec, GovConfig, ProposalActionType};
use enterprise_protocol::api::{DaoMetadata, DaoSocialData, Logo};

#[test]
fn test() -> anyhow::Result<()> {
    // TODO: rename this test
    let mut app = startup_with_versioning();

    let dao_metadata = DaoMetadata {
        name: "DAO name".to_string(),
        description: Some("DAO's description text".to_string()),
        logo: Logo::Url("www.logourl.link".to_string()),
        socials: DaoSocialData {
            github_username: Some("github handle".to_string()),
            discord_username: Some("discord handle".to_string()),
            twitter_username: Some("twitter handle".to_string()),
            telegram_username: Some("telegram handle".to_string()),
        },
    };
    let gov_config = GovConfig {
        quorum: Decimal::percent(31),
        threshold: Decimal::percent(52),
        veto_threshold: Some(Decimal::percent(17)),
        vote_duration: 250,
        minimum_deposit: Some(4u8.into()),
        allow_early_proposal_execution: false,
    };
    let dao_council = DaoCouncilSpec {
        members: vec![USER1, USER2].into_string(),
        quorum: Decimal::percent(34),
        threshold: Decimal::percent(54),
        allowed_proposal_action_types: Some(vec![
            ProposalActionType::DeployCrossChainTreasury,
            ProposalActionType::RemoveAttestation,
        ]),
    };
    let msg = CreateDaoMsg {
        dao_metadata: dao_metadata.clone(),
        gov_config: gov_config.clone(),
        dao_council: Some(dao_council.clone()),
        dao_membership: new_token_membership(default_new_token_membership()),
        asset_whitelist: Some(vec![
            cw20_unchecked(CW20_TOKEN1),
            cw20_unchecked(CW20_TOKEN2),
        ]),
        nft_whitelist: Some(vec![NFT_TOKEN1, NFT_TOKEN2].into_string()),
        minimum_weight_for_rewards: Some(6u8.into()),
        cross_chain_treasuries: None, // TODO: how do we test this?
        attestation_text: Some("Attestation text for this DAO".into()),
    };

    app.execute_contract(
        USER_DAO_CREATOR.into_addr(),
        ADDR_FACTORY.into_addr(),
        &enterprise_factory_api::msg::ExecuteMsg::CreateDao(Box::new(msg)),
        &[],
    )
    .unwrap();

    let dao_addr = get_first_dao(&app)?;

    let facade = TestFacade { app, dao_addr };
    let _components = facade.query_component_contracts()?;

    Ok(())
}
