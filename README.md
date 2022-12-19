# Enterprise Contracts
This repository contains the source code for the Enterprise protocol smart contracts on the [Terra](https://terra.money) blockchain.

You can find information about the usage and function of the smart contracts on the official Enterprise documentation [site](https://https://docs.enterprise.money/)

## Contracts

| Contract                                               | Description                                                                         |
|--------------------------------------------------------|-------------------------------------------------------------------------------------|
| [`enterprise`](./contracts/enterprise)                 | Represents a single DAO, containing all its code and treasury                       |
| [`enterprise-factory`](./contracts/enterprise-factory) | Serves as a proxy to create DAOs, storing their addresses and global configurations |

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

To deploy a contract, you can run the following command:

```
yarn terrariums deploy <contract>
```

Enterprise:
```
yarn terrariums deploy enterprise
```

Enterprise factory:
```
yarn terrariums deploy enterprise-factory
```

## License

Copyright 2022 Enterprise Protocol

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0. Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

See the License for the specific language governing permissions and limitations under the License.

