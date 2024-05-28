import task, {Deployer, Refs} from "@terra-money/terrariums";
import {Signer} from "@terra-money/terrariums/lib/src/signers";

const ENTERPRISE_FACTORY = "enterprise-factory";
const ENTERPRISE_VERSIONING = "enterprise-versioning";

const CW20_BASE = "cw20_base";
const CW721_METADATA_ONCHAIN = "cw721_metadata_onchain";

task(async ({network, deployer, signer, refs}) => {
    try {
        await deployEnterpriseFactory(refs, network, deployer, signer);
    } catch (e) {
        console.log(e);
    }
});

const deployEnterpriseFactory = async (refs: Refs, network: string, deployer: Deployer, signer: Signer): Promise<void> => {
    const enterpriseVersioning = refs.getAddress(network, ENTERPRISE_VERSIONING);
    const cw20CodeId = refs.getCodeId(network, CW20_BASE);
    const cw721CodeId = refs.getCodeId(network, CW721_METADATA_ONCHAIN);

    await deployer.storeCode(ENTERPRISE_FACTORY);
    await new Promise((resolve) => setTimeout(resolve, 5000));

    const factoryInstantiateMsg = {
        config: {
            admin: signer.key.accAddress,
            enterprise_versioning: enterpriseVersioning,
            cw20_code_id: parseInt(cw20CodeId),
            cw721_code_id: parseInt(cw721CodeId),
        },
    };

    console.log(JSON.stringify(factoryInstantiateMsg));

    try {
        await deployer.instantiate(ENTERPRISE_FACTORY, factoryInstantiateMsg, {
            admin: signer.key.accAddress,
        });
        await waitForNewBlock();
    } catch (err) {
        console.log(err);
    }

    refs.saveRefs();
}

const waitForNewBlock = async (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 5000))
