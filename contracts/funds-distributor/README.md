# Funds distributor

A contract for distributing a DAO's funds to its stakers.

Receives funds and updates indices on how many funds each user can claim.
Users can then query and claim their share of the distributed funds.

Relies on Enterprise contract to inform it of any changes in staking.


## How rewards are stored and calculated

The method for calculating a user's rewards for each individual asset is as follows:

1. A global index for the asset is tracked, denoting how many units of the asset have been rewarded per user weight since the beginning of time.
2. For each user and a given asset, we store their pending (unclaimed) rewards.
3. In addition to pending rewards, we store each user's reward index for the given asset. This index represents the last global index of the asset at which we calculated the user's pending rewards.

- Whenever we distribute new rewards for an asset, we simply increase its global index by (new amount distributed / total weight of eligible users).
- Whenever the user's weight changes, we calculate the rewards accrued since we last updated their rewards indices, and we add that to their pending rewards, setting their reward index to the global index.
- Whenever a user claims their rewards, we send them their pending rewards plus anything accrued since last calculation of their pending rewards. Then we set pending rewards to 0, and their index to current global index.