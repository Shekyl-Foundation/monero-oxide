# Shekyl monero-oxide Readiness Checklist

This checklist must pass before stressnet completion and Phase 9 audit signoff.
Any `FAIL` item is **stop-ship** for mainnet.

## Protocol Surface

| # | Gate | Status | Verification |
|---|------|--------|-------------|
| P-1 | `ProofType::FcmpPlusPlusPqc` (wire value 7) exists in `fcmp.rs` | PASS | `ProofType::try_from(7).is_ok()` |
| P-2 | `PrunableProof` with BP+, fcmp_proof, pqc_auths, reference_block | PASS | Round-trip test `fcmp_pp_prunable_round_trip` |
| P-3 | Only `FcmpPlusPlusPqc` accepted; all legacy wire values rejected | PASS | Test `proof_type_rejects_legacy_wire_values` |
| P-4 | Legacy proof types fully removed (MLSAG, Borromean, CLSAG) | PASS | No legacy code paths remain |
| P-5 | v1 transactions rejected outright | PASS | Test `v1_transaction_rejected` |
| P-6 | `SHEKYL_MIN_TX_VERSION = 2` and `SHEKYL_MIN_HF_VERSION = 1` constants exported | PASS | Compile-time check |
| P-7 | Full V2+FCMP++ transaction round-trip (serialize → deserialize → hash match) | PASS | Test `fcmp_pp_transaction_round_trip` |
| P-8 | PQC auth count mismatch rejected on deserialization | PASS | Test `fcmp_pp_pqc_auth_count_mismatch_rejected` |

## Wallet Hardening

| # | Gate | Status | Verification |
|---|------|--------|-------------|
| W-1 | No `panic!("unsupported ProofType")` in wallet send path | PASS | Manual audit of `send/tx.rs`, `send/mod.rs` |
| W-2 | `FcmpPlusPlusPqc` accepted by `SignableTransaction::validate()` | PASS | `validate()` match arm |
| W-3 | FCMP++ decoy validation skipped (no per-input ring) | PASS | `validate()` guard |
| W-4 | `SendError::FcmpSigningNotImplemented` returned when FCMP++ signing attempted | PASS | `sign()` guard |
| W-5 | `FcmpPlusPlusPqc` weight estimation with dummy proof sizes | PASS | `weight_and_necessary_fee()` |
| W-6 | View tags enabled for FCMP++ outputs | PASS | `outputs()` match arm |

## Scanner Hardening

| # | Gate | Status | Verification |
|---|------|--------|-------------|
| S-1 | No Monero-era HF upper bound (was `> 16`); replaced with `< 1` | PASS | `scan.rs` |
| S-2 | Unencrypted payment IDs always stripped (was `>= 12`); unconditional | PASS | `scan.rs` |

## CI / Test Gating

| # | Gate | Status | Verification |
|---|------|--------|-------------|
| C-1 | Monero daemon version matrix removed (`v0.17.3.2`, `v0.18.3.4`) | PASS | `tests.yml` |
| C-2 | Shekyl daemon integration (build from `feature/fcmp-plus-plus` branch) | PASS | `actions/monero/action.yml` |
| C-3 | 14 FCMP++ integration tests passing | PASS | `cargo test --package monero-oxide --test tests` |
| C-4 | Fuzz targets defined (`fuzz_rct_prunable_read`, `fuzz_transaction_read`, `fuzz_rct_base_read`) | PASS | `monero-oxide/fuzz/` |

## Security & Quality

| # | Gate | Status | Verification |
|---|------|--------|-------------|
| Q-1 | `#![deny(unsafe_code)]` on `monero-oxide` core crate | PASS | `lib.rs` |
| Q-2 | `#![deny(unsafe_code)]` on `monero-wallet` crate | PASS | `wallet/src/lib.rs` |
| Q-3 | `#![deny(unsafe_code)]` on `monero-rpc` crate | PASS | `rpc/src/lib.rs` |
| Q-4 | `#![deny(unsafe_code)]` on `monero-fcmp-plus-plus` crate | PASS | `fcmp/fcmp++/src/lib.rs` |
| Q-5 | All FCMP++ cryptographic TODOs tagged `RELEASE-BLOCKER(shekyl)` | PASS | `rg RELEASE-BLOCKER` |
| Q-6 | No `unsafe` blocks in monero-oxide workspace Rust sources (excluding target/) | PASS | Compilation with deny lint |

## Release Blockers (must resolve before audit signoff)

These items are tracked via `RELEASE-BLOCKER(shekyl)` annotations in source:

| File | Issue |
|------|-------|
| `fcmp/fcmp++/src/lib.rs` | Wrap `FCMP_PARAMS` in safe builder/accessor API |
| `fcmp/fcmp++/build.rs` | Restrict visibility of generated constants |
| `fcmp/fcmp++/src/sal/legacy_multisig.rs` | Upstream DKG offset introspection |
| `crypto/fcmps/src/gadgets/mod.rs` | Verify if on-curve constraint for `c` is redundant |
| `rpc/src/lib.rs` | Implement bulk block fetch (`get_blocks.bin`) |
| `rpc/src/lib.rs` | Implement bulk height-based fetch (`get_blocks_by_height.bin`) |
| `rpc/src/lib.rs` | Validate decoy selection against Shekyl rules |

## How to Verify

```bash
# Run the full test suite
cargo test --workspace

# Check FCMP++ specific tests
cargo test --package monero-oxide --test tests

# Verify no unsafe code
cargo check --workspace

# List all release blockers
rg 'RELEASE-BLOCKER' --glob '*.rs'

# Run fuzz targets (requires cargo-fuzz)
cd monero-oxide/fuzz && cargo +nightly fuzz run fuzz_transaction_read -- -max_total_time=300
```
