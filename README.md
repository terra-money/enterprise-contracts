# Enterprise Contracts
This repository contains the source code for the Enterprise protocol smart contracts on the [Terra](https://terra.money) blockchain.

You can find information about the usage and function of the smart contracts on the official Enterprise documentation [site](https://https://docs.enterprise.money/)

## Contracts

| Contract                                                     | Description                                                                           |
|--------------------------------------------------------------|---------------------------------------------------------------------------------------|
| [`enterprise`](./contracts/enterprise)                       | Represents a single DAO, containing all its code and treasury                         |
| [`enterprise-factory`](./contracts/enterprise-factory)       | Serves as a proxy to create DAOs, storing their addresses and global configurations   |
| [`enterprise-governance`](./contracts/enterprise-governance) | Serves as a wrapper for the poll-engine library that handles all voting-related logic |
| [`funds-distributor`](./contracts/funds-distributor)         | A contract for distributing a DAO's funds to its stakers                              |

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

### Compiling

After making sure tests pass, you can compile each contract with the following:

```sh
cargo wasm
```

#### Production

For production builds, run the following:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.6
```

This performs several optimizations which can significantly reduce the final size of the contract binaries, which will be available inside the `artifacts/` directory.

### Contract Deployment

This assumes that you already have Yarn installed (https://yarnpkg.com/).

Once you clone the repository, you can do the following;

```
yarn install
```

We are using `terrariums` to manage contract deployment. The `terrarium.json` file allows the selection of a deployment script. Deployment scripts are located in the `tasks` directory.

Terrariums synchronizes a `refs.json` file, that contains the `codeID` and `address` of deployed contracts on the various networks. This `refs.json` file can be synchronized to other locations by editing the `copy_refs_to` property in the `terrarium.json` file.

#### terrarium.json setup

To run deployment scripts, you need to first generate your own `terrarium.json` file using the provided `terrarium-template.json`.

To do that, simply replace the `pisco` and `phoenix` signers' mnemonics to one of your wallets, for testnet and mainnet respectively.
After that, just rename `terrarium-template.json` to `terrarium.json` and you're ready to run the deployment scripts.

#### Running the deployment scripts

To deploy a contract, you can run the following command:

```
yarn deploy:<contract>
```

Enterprise factory testnet:
```
yarn deploy:enterprise-factory
```

Enterprise factory mainnet:
```
yarn deploy:enterprise-factory:mainnet
```

#### Running the migration scripts

To migrate a contract, you can run the following command:

```
yarn migrate:<contract>
```

Enterprise factory testnet:
```
yarn migrate:enterprise-factory
```

Enterprise factory mainnet:
```
yarn migrate:enterprise-factory:mainnet
```

## CW20 and CW721 contracts

Enterprise is using the standard CW20 and CW721 contract implementations for tokens and NFTs.
The versions of those contracts used in different Enterprise versions are as follows:


| Enterprise contract code ID | Enterprise code version (internal incremental versioning scheme for DAOs) | CW20 version | CW721 version |
|-----------------------------|---------------------------------------------------------------------------|--------------|---------------|
| 1327                        | 2                                                                         | 1.0.1        | 0.16.0        |
| 1372                        | 3                                                                         | 1.0.1        | 0.16.0        |
| 1397                        | 4                                                                         | 1.0.1        | 0.16.0        |

## License

**Copyright 2024 Terraform Labs Pte Ltd**


Enterprise DAO is Licensed under the Apache License, Version 2.0 with Common Clause License Condition v1.0 and Additional License Condition v1.0 (the "License");


You may not use this file except in compliance with the License.


You may obtain a copy of the Apache License, Version 2.0 license at http://www.apache.org/licenses/LICENSE-2.0


Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.


See the License for the specific language governing permissions and limitations under the License.


**Commons Clause” License Condition v1.0**

The Software is provided to you by the Licensor under the License, as defined below, subject to the following condition.


Without limiting other conditions in the License, the grant of rights under the License will not include, and the License does not grant to you, the right to Sell the Software.


For purposes of the foregoing, “Sell” means practicing any or all of the rights granted to you under the License to provide to third parties or any other persons, for a fee or other monetary or non-monetary consideration (including without limitation fees for hosting or consulting/ support services related to the Software), a product or service whose value derives, entirely, substantially or similarly, from the functionality of the Software. Any license notice or attribution required by the License must also include this Commons Clause License Condition notice.


Software: Enterprise DAO


License: Apache 2.0 


Licensor: Terraform Labs Pte Ltd 


**Additional License Condition v1.0**


The terms below are in addition to the Apache 2.0 license terms and the Commons Clause License Conditions v1.0. 


Copying Restrictions:
Despite the terms of the License, no person or entity shall be permitted to copy, or reproduce in any form, any portion of the Software.


Redistribution Restrictions:
Despite the terms of the License, no person or entity shall be permitted to redistribute, share, or make publicly available any portion of the Software or derivative works thereof.



