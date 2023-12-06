use cw_orch::daemon::{networks, ChainInfo, Daemon, DaemonBuilder};
use tokio::runtime::Runtime;

use deployment::facade::deploy_facade::deploy_facade;
use deployment::logger::enable_info_logger;
use deployment::mnemonics::{use_mnemonic, DEFAULT_MNEMONIC};
use deployment::versioning::deploy_new_version::deploy_new_enterprise_version;
use deployment::versioning::deploy_versioning::deploy_versioning;

fn main() -> anyhow::Result<()> {
    enable_info_logger();

    let mnemonic = DEFAULT_MNEMONIC;
    let network = networks::PISCO_1;

    let chain = create_chain_with_default_settings(mnemonic, network)?;

    deploy_versioning(chain.clone())?;
    deploy_facade(chain.clone())?;
    deploy_new_enterprise_version(chain, 1, 0, 0, vec![])?;

    Ok(())
}

fn create_chain_with_default_settings(
    mnemonic: &str,
    network: ChainInfo,
) -> anyhow::Result<Daemon> {
    use_mnemonic(mnemonic);

    let rt = Runtime::new()?;
    let chain = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(network)
        .build()?;

    Ok(chain)
}
