To get setup:
```bash
cd contracts
yarn
```

The scripts are:
```bash
yarn run build <contract-name>
yarn run optimize <contract-name>
yarn run deploy <contract-name> --network <localterra|testnet|mainnet, default=localterra> --signer <wallet_id, default = test1>
```
For the deploy command:
```bash
yarn run deploy payment-nft
```
Will default to deploying the payment-nft contract on localterra with the inbuilt LocalTerra `test1` wallet.

Example:
```bash
yarn run deploy payment-nft --network testnet --signer test
```

For testnet and mainnet wallets, modify the `contracts/scripts/wallets.json` file.

To change the `InstantiateMsg` that a contract is deployed with, modify the `contracts/scripts/msg.json` file.