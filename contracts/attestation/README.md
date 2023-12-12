# Attestation contract

A contract representing an attestation signed by users.

Contains (possibly markup-containing) text of the attestation.
Also stores each user's individual state of attestation signature (knows whether a user signed the attestation or not).

Used by other Enterprise contracts to determine whether a user has access to certain DAO functionalities.