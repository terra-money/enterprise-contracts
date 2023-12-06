use cw_orch::daemon::Daemon;
use cw_orch::prelude::{ContractInstance, CwOrchInstantiate, CwOrchUpload};

use enterprise_factory_api::api::Config;
use enterprise_factory_api::msg::InstantiateMsg;

use crate::contracts_repository::ContractsRepository;

pub fn deploy_factory(chain: Daemon) -> anyhow::Result<()> {
    let contracts = ContractsRepository::new(chain.clone());

    let factory = contracts.factory();

    factory.upload()?;

    let admin = chain.wallet().address()?;

    factory.instantiate(
        &InstantiateMsg {
            config: Config {
                admin: admin.clone(),
                enterprise_versioning: contracts.versioning().address()?,
                // TODO: fill those in, likely from the refs file
                cw20_code_id: 0,
                cw721_code_id: 0,
            },
            global_asset_whitelist: None,
            global_nft_whitelist: None,
        },
        Some(&admin),
        None,
    )?;

    Ok(())
}
