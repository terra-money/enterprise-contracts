use cw_orch::daemon::Daemon;
use cw_orch::prelude::{CwOrchInstantiate, CwOrchUpload};

use interface::enterprise_versioning::InstantiateMsg;

use crate::contracts_repository::ContractsRepository;

pub fn deploy_versioning(chain: Daemon) -> anyhow::Result<()> {
    let contracts = ContractsRepository::new(chain.clone());

    let versioning = contracts.versioning();

    versioning.upload()?;

    let admin = chain.wallet().address()?;

    versioning.instantiate(
        &InstantiateMsg {
            admin: admin.to_string(),
        },
        Some(&admin),
        None,
    )?;

    Ok(())
}
