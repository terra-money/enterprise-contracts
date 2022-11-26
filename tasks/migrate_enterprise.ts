import task, { info } from "terrariums";

const ENTERPRISE = "enterprise";
const ENTERPRISE_FACTORY = "enterprise-factory";

task(async ({ executor, deployer, signer, refs, network }) => {
  deployer.buildContract(ENTERPRISE);
  deployer.optimizeContract(ENTERPRISE);
  const enterpriseCodeId = await deployer.storeCode(ENTERPRISE);

  const enterpriseFactory = refs.getContract(network, ENTERPRISE_FACTORY);

  try {
    await executor.execute(enterpriseFactory.address!, {
      migrate_daos: {
        new_enterprise_code_id: parseInt(enterpriseCodeId),
      },
    });
  } catch (e) {
    info(JSON.stringify(e));
  }
  info(`Migrated ${ENTERPRISE} contract.`);

  refs.saveRefs();
});
