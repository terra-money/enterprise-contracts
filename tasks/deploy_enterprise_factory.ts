import task from "terrariums";

task(async ({ deployer, signer, refs }) => {
  deployer.buildContract("enterprise");
  deployer.optimizeContract("enterprise");

  const enterpriseCodeId = await deployer.storeCode("enterprise");
  await new Promise((resolve) => setTimeout(resolve, 3000));

  const cw3CodeId = await deployer.storeCode("cw3_fixed_multisig");
  await new Promise((resolve) => setTimeout(resolve, 3000));

  const cw20CodeId = await deployer.storeCode("cw20_base");
  await new Promise((resolve) => setTimeout(resolve, 3000));

  const cw721CodeId = await deployer.storeCode("cw721_base");
  await new Promise((resolve) => setTimeout(resolve, 3000));

  deployer.buildContract("enterprise-factory");
  deployer.optimizeContract("enterprise-factory");

  await deployer.storeCode("enterprise-factory");
  await new Promise((resolve) => setTimeout(resolve, 3000));

  const instantiateMsg = {
    config: {
      enterprise_code_id: parseInt(enterpriseCodeId),
      cw3_fixed_multisig_code_id: parseInt(cw3CodeId),
      cw20_code_id: parseInt(cw20CodeId),
      cw721_code_id: parseInt(cw721CodeId),
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
