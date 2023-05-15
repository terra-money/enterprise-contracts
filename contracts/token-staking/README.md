# Token staking

A contract for managing CW20 stakes by users.

Mainly serves to:
- store user stakes and claims in a separate contract
- offload CW20 staking logic to a different contract, thus reducing the size and complexity of Enterprise contract itself