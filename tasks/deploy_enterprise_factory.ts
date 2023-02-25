import task from "terrariums";

const ENTERPRISE = "enterprise";
const ENTERPRISE_GOVERNANCE = "enterprise-governance";
const ENTERPRISE_FACTORY = "enterprise-factory";
const FUNDS_DISTRIBUTOR = "funds-distributor";

task(async ({ deployer, signer, refs }) => {
  deployer.buildContract(ENTERPRISE);
  deployer.optimizeContract(ENTERPRISE);

  const enterpriseCodeId = await deployer.storeCode(ENTERPRISE);
  await new Promise((resolve) => setTimeout(resolve, 3000));

  // deployer.buildContract(ENTERPRISE_GOVERNANCE);
  // deployer.optimizeContract(ENTERPRISE_GOVERNANCE);

  const enterpriseGovernanceCodeId = await deployer.storeCode(ENTERPRISE_GOVERNANCE);
  await new Promise((resolve) => setTimeout(resolve, 3000));

  // deployer.buildContract(FUNDS_DISTRIBUTOR);
  // deployer.optimizeContract(FUNDS_DISTRIBUTOR);

  const fundsDistributorCodeId = await deployer.storeCode(FUNDS_DISTRIBUTOR);
  await new Promise((resolve) => setTimeout(resolve, 3000));

  // const cw3CodeId = await deployer.storeCode("cw3_fixed_multisig");
  // await new Promise((resolve) => setTimeout(resolve, 3000));
  //
  // const cw20CodeId = await deployer.storeCode("cw20_base");
  // await new Promise((resolve) => setTimeout(resolve, 3000));
  //
  // const cw721CodeId = await deployer.storeCode("cw721_base");
  // await new Promise((resolve) => setTimeout(resolve, 3000));

  // deployer.buildContract(ENTERPRISE_FACTORY);
  // deployer.optimizeContract(ENTERPRISE_FACTORY);

  await deployer.storeCode(ENTERPRISE_FACTORY);
  await new Promise((resolve) => setTimeout(resolve, 3000));

  const instantiateMsg = {
    config: {
      enterprise_code_id: parseInt(enterpriseCodeId),
      enterprise_governance_code_id: parseInt(enterpriseGovernanceCodeId),
      funds_distributor_code_id: parseInt(fundsDistributorCodeId),
      cw3_fixed_multisig_code_id: parseInt("5349"),
      cw20_code_id: parseInt("5350"),
      cw721_code_id: parseInt("5351"),
    },
    global_asset_whitelist: [
      {
        native: "uluna",
      },
      {
        native:
          "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4", // axlUSDC
      },
      {
        native:
          "ibc/CBF67A2BCF6CAE343FDF251E510C8E18C361FC02B23430C121116E0811835DEF", // axlUSDT
      },
      {
        native:
          "ibc/05D299885B07905B6886F554B39346EA6761246076A1120B1950049B92B922DD", // axlWBTC
      },
      {
        native:
          "ibc/BC8A77AFBD872FDC32A348D3FB10CC09277C266CFE52081DE341C7EC6752E674", // axlWETH
      },
    ],
  };

  console.log(JSON.stringify(instantiateMsg));

  try {
    await deployer.instantiate("enterprise-factory", instantiateMsg, {
      admin: signer.key.accAddress,
    });
  } catch (err) {
    console.log(err);
  }

  refs.saveRefs();
});
