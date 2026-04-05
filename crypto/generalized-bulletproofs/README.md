# Generalized Bulletproofs

An implementation of
[Generalized Bulletproofs](</audits/generalized-bulletproofs/Security Proofs.pdf>),
a variant of the Bulletproofs arithmetic circuit statement to support Pedersen
vector commitments.

This library was audited in January, 2025 by Aaron Feickert, working for Cypher
Stack. Any subsequent changes have not undergone auditing. For reference,
please see the [audit](/audits/generalized-bulletproofs) and read the message
for the shekyl-oxide Git repository's commit
124ee09ba9d31b54b6fe4ff15d531a399bc18c99
(`git show 124ee09ba9d31b54b6fe4ff15d531a399bc18c99`).

### 'Fixed'

[This repository](https://github.com/cypherstack/generalized-bulletproofs-fix)
contains notes on a flaw within the Generalized Bulletproofs protocol,
modifying the indexes in response. This library adopts the changes to indexing
as described in that paper. These changes occured after the aforementioned
audit.
