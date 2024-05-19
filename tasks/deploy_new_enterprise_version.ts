import task, {Deployer, Executor, Refs} from "@terra-money/terrariums";
import {Signer} from "@terra-money/terrariums/lib/src/signers";

const ATTESTATION = "attestation";
const DENOM_STAKING_MEMBERSHIP = "denom-staking-membership";
const ENTERPRISE = "enterprise";
const ENTERPRISE_GOVERNANCE = "enterprise-governance";
const ENTERPRISE_GOVERNANCE_CONTROLLER = "enterprise-governance-controller";
const ENTERPRISE_TREASURY = "enterprise-treasury";
const ENTERPRISE_OUTPOSTS = "enterprise-outposts";
const ENTERPRISE_VERSIONING = "enterprise-versioning";
const FUNDS_DISTRIBUTOR = "funds-distributor";
const MULTISIG_MEMBERSHIP = "multisig-membership";
const TOKEN_STAKING_MEMBERSHIP = "token-staking-membership";
const NFT_STAKING_MEMBERSHIP = "nft-staking-membership";

task(async ({deployer, executor, network, signer, refs}) => {
    try {
        await deployNewEnterpriseVersion(refs, network, deployer, executor, 1, 2, 0);
    } catch (e) {
        console.log(e);
    }
});

const deployNewEnterpriseVersion = async (refs: Refs, network: string, deployer: Deployer, executor: Executor, major: number, minor: number, patch: number): Promise<void> => {
    await deployer.storeCode(ATTESTATION);
    await waitForNewBlock();

    await deployer.storeCode(ENTERPRISE);
    await waitForNewBlock();

    await deployer.storeCode(ENTERPRISE_GOVERNANCE);
    await waitForNewBlock();

    await deployer.storeCode(ENTERPRISE_GOVERNANCE_CONTROLLER);
    await waitForNewBlock();

    await deployer.storeCode(ENTERPRISE_TREASURY);
    await waitForNewBlock();

    await deployer.storeCode(ENTERPRISE_OUTPOSTS);
    await waitForNewBlock();

    await deployer.storeCode(FUNDS_DISTRIBUTOR);
    await waitForNewBlock();

    await deployer.storeCode(TOKEN_STAKING_MEMBERSHIP);
    await waitForNewBlock();

    await deployer.storeCode(DENOM_STAKING_MEMBERSHIP);
    await waitForNewBlock();

    await deployer.storeCode(NFT_STAKING_MEMBERSHIP);
    await waitForNewBlock();

    await deployer.storeCode(MULTISIG_MEMBERSHIP);
    await waitForNewBlock();

    const enterpriseVersioningAddr = refs.getAddress(network, ENTERPRISE_VERSIONING);

    try {
        await executor.execute(enterpriseVersioningAddr, {
            add_version: {
                version: {
                    version: {
                        major: major,
                        minor: minor,
                        patch: patch,
                    },
                    changelog: [],
                    attestation_code_id: parseInt(refs.getCodeId(network, ATTESTATION)),
                    enterprise_code_id: parseInt(refs.getCodeId(network, ENTERPRISE)),
                    enterprise_governance_code_id: parseInt(refs.getCodeId(network, ENTERPRISE_GOVERNANCE)),
                    enterprise_governance_controller_code_id: parseInt(refs.getCodeId(network, ENTERPRISE_GOVERNANCE_CONTROLLER)),
                    enterprise_treasury_code_id: parseInt(refs.getCodeId(network, ENTERPRISE_TREASURY)),
                    enterprise_outposts_code_id: parseInt(refs.getCodeId(network, ENTERPRISE_OUTPOSTS)),
                    funds_distributor_code_id: parseInt(refs.getCodeId(network, FUNDS_DISTRIBUTOR)),
                    token_staking_membership_code_id: parseInt(refs.getCodeId(network, TOKEN_STAKING_MEMBERSHIP)),
                    denom_staking_membership_code_id: parseInt(refs.getCodeId(network, DENOM_STAKING_MEMBERSHIP)),
                    nft_staking_membership_code_id: parseInt(refs.getCodeId(network, NFT_STAKING_MEMBERSHIP)),
                    multisig_membership_code_id: parseInt(refs.getCodeId(network, MULTISIG_MEMBERSHIP)),
                }
            }
        })
    } catch (e) {
        console.log(e);
    }

    refs.saveRefs();
}

const waitForNewBlock = async (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 5000))
