use crate::api::CreateProposalMsg;
use cosmwasm_std::{from_json, to_json_string, StdResult};

#[test]
fn stuff() -> StdResult<()> {
    let json = to_json_string(&CreateProposalMsg {
        title: "123".to_string(),
        description: Some("stuff".to_string()),
        proposal_actions: vec![],
        deposit_owner: Some("stranger".to_string()),
    })?;

    println!("{}", json);

    let structure: CreateProposalMsg = from_json(json)?;

    assert_eq!(
        structure,
        CreateProposalMsg {
            title: "123".to_string(),
            description: Some("stuff".to_string()),
            proposal_actions: vec![],
            deposit_owner: Some("stranger".to_string()),
        }
    );

    Ok(())
}
