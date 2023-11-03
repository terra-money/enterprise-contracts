use cosmwasm_std::{to_json_binary, Empty};
use enterprise_protocol::error::DaoResult;
use enterprise_protocol::msg::MigrateMsg;

#[test]
fn initial_test() -> DaoResult<()> {
    assert_eq!(2 + 2, 4);

    let serde: MigrateMsg = serde_json_wasm::from_str("{}").unwrap();

    let msg = serde_json_wasm::to_string(&Empty {}).unwrap();
    println!("{}", msg);

    let msg = serde_json_wasm::to_string(&MigrateMsg {}).unwrap();
    println!("{}", msg);

    let serde2: MigrateMsg = serde_json_wasm::from_str(&msg).unwrap();

    println!("equal: {}", serde == serde2);

    println!(
        "equal 2: {}",
        to_json_binary(&MigrateMsg {}).unwrap() == to_json_binary(&Empty {}).unwrap()
    );

    Ok(())
}
