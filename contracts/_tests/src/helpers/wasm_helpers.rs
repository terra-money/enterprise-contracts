use cosmwasm_std::Addr;
use cw_multi_test::App;

pub fn assert_addr_code_id(app: &App, contract: &Addr, code_id: u64) {
    assert_eq!(code_id, app.contract_data(contract).unwrap().code_id as u64)
}

pub fn assert_contract_admin(app: &App, contract: &Addr, admin: &str) {
    assert_eq!(
        Some(admin.to_string()),
        app.contract_data(contract)
            .unwrap()
            .admin
            .map(|it| it.to_string()),
    )
}
