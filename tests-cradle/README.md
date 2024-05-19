### Cradle tests

Tests using a fork of mainnet to simulate DAOs upgrading to new versions.

#### Env setup

To run the tests, an `.env` file in the root directory of the repository is required.

The file requires the following setup:

```
NETWORK=<mainnet|testnet>
LCD_ENDPOINT=<Cradle LCD endpoint, can be found in your Cradle dashboard>
JWT_TOKEN=<JWT token for your Cradle session>
CHAIN_ID=<e.g. phoenix-1>
MNEMONIC_KEY=<mnemonic of the wallet used to interact with the chain>
```

#### Running

To run the tests, run (from the root directory of the repository):

```sh
yarn tests-cradle
```