# Enterprise governance controller

A contract that binds the logic of enterprise-governance contract together with
membership contracts, and executes proposals centrally, dispatching messages to other contracts as needed.

This contract has privileges over other contracts (it's set to their admin, usually) that allows it to control all their
settings and privileged behaviors.

The contract contains several big pieces of functionality:

- Proposal meta-data (proposal actions, which membership type it is associated with, etc.)
- General-members-type governance (creating proposals, voting on them, and executing them)
- Council-type governance, where a council of select members is defined to run specific types of proposals