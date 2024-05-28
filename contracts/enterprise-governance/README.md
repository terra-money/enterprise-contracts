# Enterprise governance

A contract for managing Enterprise's polls.
Essentially a proxy to the poll-engine library.

Mainly serves to:

- create and store essential proposal data (proposer, voting period, quorum, threshold)
- calculate appropriate proposal status when trying to execute or query it.

Does not validate who can vote or calculate their voting power.