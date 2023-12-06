use cw_orch::daemon::Daemon;
use cw_orch::prelude::{ContractInstance, CwOrchInstantiate, CwOrchUpload};

use interface::enterprise_facade::InstantiateMsg as InstantiateFacadeMsg;
use interface::enterprise_facade_v1::InstantiateMsg as InstantiateFacadeV1Msg;
use interface::enterprise_facade_v2::InstantiateMsg as InstantiateFacadeV2Msg;

use crate::contracts_repository::ContractsRepository;

pub fn deploy_facade(chain: Daemon) -> anyhow::Result<()> {
    let contracts = ContractsRepository::new(chain.clone());

    let versioning = contracts.versioning();

    let facade_v1 = contracts.facade_v1();
    facade_v1.upload()?;
    facade_v1.instantiate(
        &InstantiateFacadeV1Msg {
            enterprise_versioning: versioning.addr_str()?,
        },
        Some(&chain.wallet().address()?),
        None,
    )?;

    let facade_v2 = contracts.facade_v2();
    facade_v2.upload()?;
    facade_v2.instantiate(
        &InstantiateFacadeV2Msg {},
        Some(&chain.wallet().address()?),
        None,
    )?;

    let facade = contracts.facade();
    facade.upload()?;
    facade.instantiate(
        &InstantiateFacadeMsg {
            enterprise_facade_v1: facade_v1.addr_str()?,
            enterprise_facade_v2: facade_v2.addr_str()?,
        },
        Some(&chain.wallet().address()?),
        None,
    )?;

    Ok(())
}
