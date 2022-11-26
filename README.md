# Getting Started

This assumes that you already have Yarn installed (https://yarnpkg.com/).

Once you clone the repository, you can do the following;

```
yarn install
yarn run apps:start
```

## Contract Deployment

We are now using `terrariums` to manage contract deployment. The `terrarium.json` file allows the selection of a deployment script, or the use of a static `instantiate_msg` for contract deployments. Simple contract deployments can simply specify this `instantiate_msg` in the `terrarium.json` file, and not worry about creating a deployment script. Deployment scripts are located in the `tasks` directory.

Terrariums synchronizes a `refs.json` file, that contains the `codeID` and `address` of deployed contracts on the various networks. This `refs.json` file can be synchronized to other locations by editing the `copy_refs_to` property in the `terrarium.json` file.

To deploy a contract, you can run the following command:

```
yarn terrariums deploy <contract>
```

### Payment Protocol Deployment

The `tasks/deploy_payment_minter.ts` script manages the deployment of the payment protocol. It can be run by: `yarn terrariums deploy payment-minter`.

### Enterprise Deployment

The `tasks/deploy_enterprise.ts` script manages the deployment of the Enterprise contract. It can be run by: `yarn terrariums deploy enterprise`.

**Note:** You must first deploy the payment protocol using its deploy task.
