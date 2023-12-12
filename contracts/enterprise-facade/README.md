# Enterprise facade

This contract retains the original Enterprise's query API and several execute messages that do not take into
account the sender.

It allows the frontend and indexers to retain the old API, even though the structure of Enterprise contracts has been
significantly revamped since the original.

This contract will simply take an Enterprise address, determine which version it is, and forward the calls to an
appropriate version of another facade contract.