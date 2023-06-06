# NFT staking membership

A contract for managing an NFT (CW721) staking membership for an Enterprise DAO.
Essentially a proxy to the nft-staking library.

Mainly serves to:
- store users' NFT stakes
- provide an interface to stake, unstake, and claim user NFTs
- provide queries for user and total stakes, and user claims