# shekyl-oxide (Shekyl fork)

This is the Shekyl fork of shekyl-oxide, implementing FCMP++ (Full-Chain
Membership Proofs) support for the Shekyl protocol. The library provides
Rust-native types and serialization for the full Shekyl transaction format.

Originally forked from the upstream shekyl-oxide project, this fork adds:

- **FCMP++ proof type** (`RCTTypeFcmpPlusPlusPqc = 7`) as the only accepted
  proof type for Shekyl consensus
- **Shekyl policy enforcement** rejecting legacy Monero proof types from genesis
- **Post-quantum authentication** (ML-DSA-65 per-input signatures)
- **`#![deny(unsafe_code)]`** across all critical crates
- **Comprehensive FCMP++ test suite** and fuzz targets

The library primarily provides two crate families:

- [`shekyl-oxide`](./shekyl-oxide): Shekyl transaction protocol types and
  serialization.
- [`shekyl-wallet`](./shekyl-oxide/wallet): Wallet functionality (scanning,
  transaction construction).

### Readiness Status

See [`SHEKYL_READINESS.md`](./SHEKYL_READINESS.md) for the full pass/fail
checklist required before stressnet and audit signoff.

We welcome contributions, either via making issues or opening pull requests.
For the latter, please see our [contributing guidelines](./Contributing.md).

To report a security issue, please see our [disclosure policy](./Security.md).
