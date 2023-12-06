use cw_orch::daemon::Daemon;
use cw_orch::prelude::{ContractInstance, CwOrchExecute, CwOrchUpload};

use interface::enterprise_versioning::{AddVersionMsg, ExecuteMsg, Version, VersionInfo};

use crate::contracts_repository::ContractsRepository;

pub fn deploy_new_enterprise_version(
    chain: Daemon,
    major: u64,
    minor: u64,
    patch: u64,
    changelog: Vec<String>,
) -> anyhow::Result<()> {
    let contracts = ContractsRepository::new(chain.clone());

    contracts.attestation().upload()?;
    contracts.denom_staking_membership().upload()?;
    contracts.enterprise().upload()?;
    contracts.governance().upload()?;
    contracts.governance_controller().upload()?;
    contracts.outposts().upload()?;
    contracts.treasury().upload()?;
    contracts.versioning().upload()?;
    contracts.funds_distributor().upload()?;
    contracts.multisig_membership().upload()?;
    contracts.nft_staking_membership().upload()?;
    contracts.token_staking_membership().upload()?;

    contracts.versioning().execute(
        &ExecuteMsg::AddVersion(AddVersionMsg {
            version: VersionInfo {
                version: Version {
                    major,
                    minor,
                    patch,
                },
                changelog,
                attestation_code_id: contracts.attestation().code_id()?,
                enterprise_code_id: contracts.enterprise().code_id()?,
                enterprise_governance_code_id: contracts.governance().code_id()?,
                enterprise_governance_controller_code_id: contracts
                    .governance_controller()
                    .code_id()?,
                enterprise_outposts_code_id: contracts.attestation().code_id()?,
                enterprise_treasury_code_id: contracts.treasury().code_id()?,
                funds_distributor_code_id: contracts.funds_distributor().code_id()?,
                token_staking_membership_code_id: contracts.token_staking_membership().code_id()?,
                denom_staking_membership_code_id: contracts.denom_staking_membership().code_id()?,
                nft_staking_membership_code_id: contracts.nft_staking_membership().code_id()?,
                multisig_membership_code_id: contracts.multisig_membership().code_id()?,
            },
        }),
        None,
    )?;

    Ok(())
}
