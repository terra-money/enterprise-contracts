# Funds distributor

A contract for distributing a DAO's funds to its stakers.

Receives funds and updates indices on how many funds each user can claim.
Users can then query and claim their share of the distributed funds.

Does not store user stakes.
Relies on Enterprise contract to inform it of any changes in staking.