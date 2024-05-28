import task, {Deployer, Refs} from "@terra-money/terrariums";
import {Signer} from "@terra-money/terrariums/lib/src/signers";

const ENTERPRISE_VERSIONING = "enterprise-versioning";
const ENTERPRISE_FACADE = "enterprise-facade";
const ENTERPRISE_FACADE_V1 = "enterprise-facade-v1";
const ENTERPRISE_FACADE_V2 = "enterprise-facade-v2";

task(async ({deployer, network, signer, refs}) => {
    try {
        await deployEnterpriseFacade(refs, network, deployer, signer);
    } catch (e) {
        console.log(e);
    }
});

const deployEnterpriseFacade = async (refs: Refs, network: string, deployer: Deployer, signer: Signer): Promise<void> => {
    await deployer.storeCode(ENTERPRISE_FACADE_V1);
    await waitForNewBlock();
    await deployer.storeCode(ENTERPRISE_FACADE_V2);
    await waitForNewBlock();

    await deployer.storeCode(ENTERPRISE_FACADE);
    await waitForNewBlock();

    try {
        await deployer.instantiate(ENTERPRISE_FACADE_V1, {
                enterprise_versioning: refs.getAddress(network, ENTERPRISE_VERSIONING),
            },
            {
                admin: signer.key.accAddress,
                label: "Enterprise facade V1",
            });
        await waitForNewBlock();

        await deployer.instantiate(ENTERPRISE_FACADE_V2, {},
            {
                admin: signer.key.accAddress,
                label: "Enterprise facade V2",
            });

        refs.saveRefs();

        await deployer.instantiate(ENTERPRISE_FACADE, {
                enterprise_facade_v1: refs.getAddress(network, ENTERPRISE_FACADE_V1),
                enterprise_facade_v2: refs.getAddress(network, ENTERPRISE_FACADE_V2),
            },
            {
                admin: signer.key.accAddress,
                label: "Enterprise facade",
            });

    } catch (err) {
        console.log(err);
    }

    refs.saveRefs();
}

const waitForNewBlock = async (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 5000))
