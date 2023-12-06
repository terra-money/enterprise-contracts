use cw_orch::daemon::Daemon;

use interface::attestation::AttestationContract;
use interface::denom_staking_membership::DenomStakingMembershipContract;
use interface::enterprise::EnterpriseContract;
use interface::enterprise_facade::EnterpriseFacadeContract;
use interface::enterprise_facade_v1::EnterpriseFacadeV1Contract;
use interface::enterprise_facade_v2::EnterpriseFacadeV2Contract;
use interface::enterprise_factory::EnterpriseFactoryContract;
use interface::enterprise_governance::EnterpriseGovernanceContract;
use interface::enterprise_governance_controller::EnterpriseGovernanceControllerContract;
use interface::enterprise_outposts::EnterpriseOutpostsContract;
use interface::enterprise_treasury::EnterpriseTreasuryContract;
use interface::enterprise_versioning::EnterpriseVersioningContract;
use interface::funds_distributor::FundsDistributorContract;
use interface::multisig_membership::MultisigMembershipContract;
use interface::nft_staking_membership::NftStakingMembershipContract;
use interface::token_staking_membership::TokenStakingMembershipContract;

const ATTESTATION_ID: &str = "attestation";
const DENOM_STAKING_MEMBERSHIP_ID: &str = "denom_staking_membership";
const ENTERPRISE_ID: &str = "enterprise";
const FACTORY_ID: &str = "enterprise_factory";
const FACADE_V1_ID: &str = "enterprise_facade_v1";
const FACADE_V2_ID: &str = "enterprise_facade_v2";
const FACADE_ID: &str = "enterprise_facade";
const GOVERNANCE_ID: &str = "enterprise_governance";
const GOVERNANCE_CONTROLLER_ID: &str = "enterprise_governance_controller";
const OUTPOSTS_ID: &str = "enterprise_outposts";
const TREASURY_ID: &str = "enterprise_treasury";
const FUNDS_DISTRIBUTOR_ID: &str = "funds_distributor";
const MULTISIG_MEMBERSHIP_ID: &str = "multisig_membership";
const NFT_STAKING_MEMBERSHIP_ID: &str = "nft_staking_membership";
const TOKEN_STAKING_MEMBERSHIP_ID: &str = "token_staking_membership";
const VERSIONING_ID: &str = "enterprise_versioning";

pub struct ContractsRepository {
    chain: Daemon,
}

impl ContractsRepository {
    pub fn new(chain: Daemon) -> ContractsRepository {
        ContractsRepository { chain }
    }

    pub fn attestation(&self) -> AttestationContract<Daemon> {
        AttestationContract::new(ATTESTATION_ID, self.chain.clone())
    }

    pub fn denom_staking_membership(&self) -> DenomStakingMembershipContract<Daemon> {
        DenomStakingMembershipContract::new(DENOM_STAKING_MEMBERSHIP_ID, self.chain.clone())
    }

    pub fn enterprise(&self) -> EnterpriseContract<Daemon> {
        EnterpriseContract::new(ENTERPRISE_ID, self.chain.clone())
    }

    pub fn facade(&self) -> EnterpriseFacadeContract<Daemon> {
        EnterpriseFacadeContract::new(FACADE_ID, self.chain.clone())
    }

    pub fn facade_v1(&self) -> EnterpriseFacadeV1Contract<Daemon> {
        EnterpriseFacadeV1Contract::new(FACADE_V1_ID, self.chain.clone())
    }

    pub fn facade_v2(&self) -> EnterpriseFacadeV2Contract<Daemon> {
        EnterpriseFacadeV2Contract::new(FACADE_V2_ID, self.chain.clone())
    }

    pub fn factory(&self) -> EnterpriseFactoryContract<Daemon> {
        EnterpriseFactoryContract::new(FACTORY_ID, self.chain.clone())
    }

    pub fn governance(&self) -> EnterpriseGovernanceContract<Daemon> {
        EnterpriseGovernanceContract::new(GOVERNANCE_ID, self.chain.clone())
    }

    pub fn governance_controller(&self) -> EnterpriseGovernanceControllerContract<Daemon> {
        EnterpriseGovernanceControllerContract::new(GOVERNANCE_CONTROLLER_ID, self.chain.clone())
    }

    pub fn outposts(&self) -> EnterpriseOutpostsContract<Daemon> {
        EnterpriseOutpostsContract::new(OUTPOSTS_ID, self.chain.clone())
    }

    pub fn treasury(&self) -> EnterpriseTreasuryContract<Daemon> {
        EnterpriseTreasuryContract::new(TREASURY_ID, self.chain.clone())
    }

    pub fn funds_distributor(&self) -> FundsDistributorContract<Daemon> {
        FundsDistributorContract::new(FUNDS_DISTRIBUTOR_ID, self.chain.clone())
    }

    pub fn multisig_membership(&self) -> MultisigMembershipContract<Daemon> {
        MultisigMembershipContract::new(MULTISIG_MEMBERSHIP_ID, self.chain.clone())
    }

    pub fn nft_staking_membership(&self) -> NftStakingMembershipContract<Daemon> {
        NftStakingMembershipContract::new(NFT_STAKING_MEMBERSHIP_ID, self.chain.clone())
    }

    pub fn token_staking_membership(&self) -> TokenStakingMembershipContract<Daemon> {
        TokenStakingMembershipContract::new(TOKEN_STAKING_MEMBERSHIP_ID, self.chain.clone())
    }

    pub fn versioning(&self) -> EnterpriseVersioningContract<Daemon> {
        EnterpriseVersioningContract::new(VERSIONING_ID, self.chain.clone())
    }
}
