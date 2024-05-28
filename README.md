# Enterprise Contracts

This repository contains the source code for the Enterprise protocol smart contracts on the [Terra](https://terra.money)
blockchain.

You can find information about the usage and function of the smart contracts on the official Enterprise
documentation [site](https://docs.enterprise.money/).

## Contracts

| Contract                                                                           | Description                                                                                                                                                           |
|------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [`denom-staking-membership`](./contracts/denom-staking-membership)                 | Membership contract for DAOs that base their membership on staking of native denoms.                                                                                  |
| [`enterprise`](./contracts/enterprise)                                             | On-chain admin (has migration rights) of all other DAO's contracts; holds addresses of all other DAO contracts.                                                       |
| [`enterprise-facade`](./contracts/enterprise-facade)                               | Facade that exposes functions in a way that pre-1.0.0 DAOs did, enabling frontend to interact with all DAOs the same way (using the facade).                          |
| [`enterprise-facade-v1`](./contracts/enterprise-facade-v1)                         | Facade implementation for pre-1.0.0 DAOs                                                                                                                              |
| [`enterprise-facade-v2`](./contracts/enterprise-facade-v2)                         | Facade implementation for 1.0.0 and later DAOs                                                                                                                        |
| [`enterprise-factory`](./contracts/enterprise-factory)                             | A proxy to create DAOs, storing their addresses, and global configurations.                                                                                           |
| [`enterprise-governance`](./contracts/enterprise-governance)                       | A wrapper for the poll-engine library that handles all voting-related logic.                                                                                          |
| [`enterprise-governance-controller`](./contracts/enterprise-governance-controller) | Validates who and under what circumstances can create proposals, vote on them, and execute them; reports governance actions; has privileges over other DAO contracts. |
| [`enterprise-outposts`](./contracts/enterprise-outposts)                           | Deals with cross-chain treasuries - keeps references to them, and communicates with them.                                                                             |
| [`enterprise-treasury`](./contracts/enterprise-treasury)                           | Holds all the DAO's funds (excluding proposal deposits). Acts as the main address of the DAO.                                                                         |
| [`enterprise-versioning`](./contracts/enterprise-versioning)                       | Global contract that maps contract code IDs to DAO versions.                                                                                                          |
| [`funds-distributor`](./contracts/funds-distributor)                               | Distributes funds (rewards) to DAO's members.                                                                                                                         |
| [`ics721-callback-proxy`](./contracts/ics721-callback-proxy)                       | Global contract that accepts callbacks from incoming ICS721 NFTs and passes them along.                                                                               |
| [`multisig-membership`](./contracts/multisig-membership)                           | Membership contract for DAOs that base their membership on a multisig.                                                                                                |
| [`nft-staking-membership`](./contracts/nft-staking-membership)                     | Membership contract for DAOs that base their membership on staking of CW721 NFTs.                                                                                     |
| [`token-staking-membership`](./contracts/token-staking-membership)                 | Membership contract for DAOs that base their membership on staking of CW20 tokens.                                                                                    |

## Development

### Environment Setup

- Rust v1.44.1+
- `wasm32-unknown-unknown` target
- Docker

1. Install `rustup` via https://rustup.rs/

2. Run the following:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

3. Make sure [Docker](https://www.docker.com/) is installed

### Unit / Integration Tests

Each contract contains Rust unit and integration tests embedded within the contract source directories. You can run:

```sh
cargo test
```

#### Compiling and building contracts

For production builds, run the following:

```sh
./build.sh
```

This performs several optimizations which can significantly reduce the final size of the contract binaries, which will
be available inside the `artifacts/` directory.

#### Faster builds for ARM

While developing and testing on Apple silicon or other ARM architectures, you can build contract .wasm files using:

```sh
./build_arm.sh
```

### Contract Deployment

Contracts have deployment scripts that make it easier to publish them to various networks.

#### Yarn + Terrariums setup

This assumes that you already have Yarn installed (https://yarnpkg.com/).

Once you clone the repository, you can do the following;

```
yarn install
```

We are using `terrariums` to manage contract deployment. The `terrarium.json` file allows the selection of a deployment
script. Deployment scripts are located in the `tasks` directory.

Terrariums synchronizes a `refs.json` file, that contains the `codeID` and `address` of deployed contracts on the
various networks.

#### terrarium.json setup

To run deployment scripts, you need to first generate your own `terrarium.json` file using the
provided `terrarium-template.json`.

To do that, simply replace the signers' mnemonics to one of your wallets, for the networks you plan to use.

After that, just rename `terrarium-template.json` to `terrarium.json` and you're ready to run the deployment scripts.

#### Running the deployment scripts

To deploy a contract, you can run the following command:

```
yarn deploy:<contract>:<mainnet|testnet|staging|cradle>
```

Enterprise factory testnet:

```
yarn deploy:enterprise-factory:testnet
```

Enterprise factory mainnet:

```
yarn deploy:enterprise-factory:mainnet
```

#### Deployment sequence

The contracts have certain interdependencies that require specific sequence in contract deployment.

The correct deployment sequence:

1. enterprise-versioning (global instance)
2. enterprise-factory (global instance)
3. enterprise-facade (global instance)
4. New Enterprise version (will deploy code for each of the DAO-specific contracts, and package them into a version
   denoted in the `tasks/deploy_enterprise_versioning.ts` file).

#### Running the migration scripts

To migrate a contract, you can run commands similar to deployment, however you will use `migrate` instead of `deploy`:

```
yarn migrate:<contract>:<mainnet|testnet|staging|cradle>
```

Enterprise factory testnet:

```
yarn migrate:enterprise-factory:testnet
```

### Cradle tests

A suite of tests using [NewMetric's](https://www.newmetric.xyz/) Cradle has been created, and can be found
under [tests-cradle/](./tests-cradle).

#### Purpose

The tests are designed to execute real transactions on a fork of mainnet.

These are used as 'smoke tests' for new versions of Enterprise. They allow developers to exactly simulate existing DAOs
upgrading to new versions and check whether basic functions (like proposals, or upgrading again) work, so that DAOs are
not bricked after an update.

#### Env setup for Cradle tests

To run the tests, an `.env` file in the root directory of the repository is required.

The file requires the following setup:

```
NETWORK=<mainnet|testnet>
LCD_ENDPOINT=<Cradle LCD endpoint, can be found in your Cradle dashboard>
JWT_TOKEN=<JWT token for your Cradle session>
CHAIN_ID=<e.g. phoenix-1>
MNEMONIC_KEY=<mnemonic of the wallet used to interact with the chain>
```

#### Running Cradle tests

Before running Cradle tests, you need a valid setup in the form of an `.env` file, described above.

To run the tests, run:

```sh
yarn tests-cradle
```

## License

**Copyright 2024 Terraform Labs Pte Ltd**

Enterprise DAO is Licensed under the Apache License, Version 2.0 with Common Clause License Condition v1.0 and
Additional License Condition v1.0 (the "License");

You may not use this file except in compliance with the License.

You may obtain a copy of the Apache License, Version 2.0 license at http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "
AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

See the License for the specific language governing permissions and limitations under the License.

**Commons Clause” License Condition v1.0**

The Software is provided to you by the Licensor under the License, as defined below, subject to the following condition.

Without limiting other conditions in the License, the grant of rights under the License will not include, and the
License does not grant to you, the right to Sell the Software.

For purposes of the foregoing, “Sell” means practicing any or all of the rights granted to you under the License to
provide to third parties or any other persons, for a fee or other monetary or non-monetary consideration (including
without limitation fees for hosting or consulting/ support services related to the Software), a product or service whose
value derives, entirely, substantially or similarly, from the functionality of the Software. Any license notice or
attribution required by the License must also include this Commons Clause License Condition notice.

Software: Enterprise DAO

License: Apache 2.0

Licensor: Terraform Labs Pte Ltd

**Additional License Condition v1.0**

The terms below are in addition to the Apache 2.0 license terms and the Commons Clause License Conditions v1.0.

Copying Restrictions:
Despite the terms of the License, no person or entity shall be permitted to copy, or reproduce in any form, any portion
of the Software.

Redistribution Restrictions:
Despite the terms of the License, no person or entity shall be permitted to redistribute, share, or make publicly
available any portion of the Software or derivative works thereof.



