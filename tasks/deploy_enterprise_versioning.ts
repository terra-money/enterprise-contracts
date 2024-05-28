import task, {Deployer, Refs} from "@terra-money/terrariums";
import {Signer} from "@terra-money/terrariums/lib/src/signers";

const ENTERPRISE_VERSIONING = "enterprise-versioning";

task(async ({deployer, signer, refs}) => {
    try {
        await deployEnterpriseVersioning(refs, deployer, signer);
    } catch (e) {
        console.log(e);
    }
});

export const deployEnterpriseVersioning = async (refs: Refs, deployer: Deployer, signer: Signer): Promise<void> => {
    await deployer.storeCode(ENTERPRISE_VERSIONING);
    await waitForNewBlock();

    const versioningInstantiateMsg = {
        admin: signer.key.accAddress,
    };

    try {
        await deployer.instantiate(ENTERPRISE_VERSIONING, versioningInstantiateMsg, {
            admin: signer.key.accAddress,
            label: "Enterprise versioning",
        });
    } catch (err) {
        console.log(err);
    }

    refs.saveRefs();
}

const waitForNewBlock = async (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 5000))
