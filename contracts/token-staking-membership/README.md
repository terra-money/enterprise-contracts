# Token staking membership

A contract for managing a token (CW20) staking membership for an Enterprise DAO.
Essentially a proxy to the token-staking library.

Mainly serves to:
- store users' token stakes
- provide an interface to stake, unstake, and claim user tokens
- provide queries for user and total weights, and user claims