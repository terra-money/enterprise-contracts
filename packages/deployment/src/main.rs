use cw_orch::prelude::*;
use tokio::runtime::Runtime;
use interface::enterprise_treasury::{EnterpriseTreasuryContract, InstantiateMsg};

fn main() {
    dotenv::dotenv().ok();
    let env = dotenv::dotenv();
    pretty_env_logger::init();
    let runtime = Runtime::new().unwrap();

    let daemon = DaemonBuilder::default()
        .chain(cw_orch::daemon::networks::PISCO_1)
        .handle(runtime.handle())
        .build()
        .unwrap();

    let treasury = EnterpriseTreasuryContract::new("local:treasury", daemon.clone());
    let upload_res = treasury.upload();
    assert!(upload_res.is_ok());
    let instantiate_res = treasury.instantiate(
        &InstantiateMsg {
            admin: "terra1a8dxkrapwj4mkpfnrv7vahd0say0lxvd0ft6qv".to_string(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    None,
    None);
    assert!(instantiate_res.is_ok());
}