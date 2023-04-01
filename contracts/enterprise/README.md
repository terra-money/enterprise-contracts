# Enterprise

Enterprise contract represents a single DAO and holds all its belonging assets (staked governance assets, proposal deposits, and treasury).

The contract contains several big pieces of functionality:
- Membership management and queries
- General-members-type governance (creating proposals, voting on them, and executing them) and staking of governance assets
- Council-type governance, where a council of select members is defined to run specific types of proposals
- Treasury definition and queries

## Dependencies

The contract relies on reference [CW20](https://github.com/CosmWasm/cw-plus/tree/main/contracts/cw20-base) and [CW721](https://github.com/CosmWasm/cw-nfts/tree/main/contracts/cw721-base) implementations to create and run token and NFT DAOs, respectively.