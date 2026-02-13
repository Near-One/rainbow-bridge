## Overview

Rainbow Bridge ETH→NEAR light client. An on-chain Ethereum light client contract on NEAR that tracks Ethereum beacon chain finality via sync committee updates, plus a relayer that feeds it data.

## Repository Layout

Two Cargo workspaces plus a standalone crate:

- **`contracts/near/`** — NEAR smart contracts workspace (`eth2-client`, `eth-prover`, `eth-types`)
- **`eth2near/`** — Legacy relayer workspace (7 crates)
- **`relayer/`** — New relayer (standalone, Rust edition 2024)

The eth2 light client lives in `contracts/near/eth2-client/`. Supporting crates:
- `eth-types` — Ethereum execution/consensus type definitions (BlockHeader, BeaconBlockHeader, SyncCommittee, etc.)
- `eth2-utility` — Consensus logic: network configs, fork handling, merkle proof verification, sync committee period computation
- `eth2_hashing` — WASM-compatible Keccak256 (patched ethereum_hashing for NEAR SDK)

## Build, Test, Lint

All contract commands run from `contracts/near/`:

```bash
# Build (requires cargo-near and wasm32-unknown-unknown target)
cargo near build reproducible-wasm --manifest-path contracts/near/eth2-client/Cargo.toml

# Test (the canonical CI sequence — builds test WASM first, then runs tests twice: with and without default features)
cd contracts/near/eth2-client && ./test.sh

# Run all tests
cd contracts/near && cargo test -p eth2-client

# Run a single test
cd contracts/near && cargo test -p eth2-client test_gc_headers -- --nocapture

# Unit tests only (no mainnet feature, allows validate_updates=false)
cd contracts/near && cargo test -p eth2-client --no-default-features -- --nocapture

# Lint
cd contracts/near && cargo clippy
cd contracts/near && cargo fmt --check
```

**Feature flags** (eth2-client): `default = ["logs", "mainnet"]`
- `mainnet` — enforces `validate_updates` and `verify_bls_signatures` on init
- `logs` — enables `env::log_str()` output
- Unit tests marked `#[cfg(not(feature = "mainnet"))]` only run with `--no-default-features`

**CI** (`.github/workflows/contracts-near.yml`): runs `make build-eth2-client`, `./test.sh`, and relayer tests. Requires `git-lfs` for test data.

## Contract State Machine

The eth2-client alternates between two modes:

```
SubmitLightClientUpdate → submit_beacon_chain_light_client_update()
    validates sync committee signatures, merkle proofs, period transitions
    updates finalized beacon header + sync committees
    → transitions to SubmitHeader

SubmitHeader → submit_execution_header() (called repeatedly)
    accepts execution blocks in reverse order (newest→oldest)
    stores block_number→block_hash in LookupMap
    GCs old blocks beyond hashes_gc_threshold
    when chain is closed (reaches previous finalized block + 1)
    → transitions back to SubmitLightClientUpdate
```

Light client updates can advance by at most 1 sync committee period (8192 slots ≈ 27 hours). This bounds GC to at most one period's worth of removals per cycle.

## Key Protocol Constants

From `eth2-utility/src/consensus.rs`:
- `SLOTS_PER_EPOCH = 32` (~6.4 min)
- `EPOCHS_PER_SYNC_COMMITTEE_PERIOD = 256`
- One sync committee period = 8192 slots ≈ 27 hours
- `hashes_gc_threshold` is typically 51000 (~7 days of blocks)

## Storage Keys

```rust
StorageKey::FinalizedExecutionBlocks  // LookupMap<u64, H256> — block number → hash
StorageKey::FinalizedExecutionHeader  // LazyOption<ExecutionHeaderInfo>
StorageKey::CurrentSyncCommittee      // LazyOption<SyncCommittee>
StorageKey::NextSyncCommittee         // LazyOption<SyncCommittee>
```

`__DeprecatedUnfinalizedHeaders` and `__DeprecatedSubmitters` are leftover from V1 migration — do not reuse.

## Access Control

Uses `near-plugins` for roles: `DAO`, `PauseManager`, `UpgradableCodeStager/Deployer`, `UnrestrictedSubmitLightClientUpdate`, `UnrestrictedSubmitExecutionHeader`. Admin methods (`update_trusted_signer`, `update_hashes_gc_threshold`, etc.) require `Role::DAO`.

## Test Data

Test data lives in `contracts/near/eth2-client/src/data/` (LFS-tracked JSON files). Sepolia is the primary test network. Regenerate with:
```bash
cd contracts/near/eth2-client/src/data && uv run dump_sepolia_data.py
```

## Migration Pattern

`migrate.rs` defines the previous struct version (e.g., `Eth2ClientV1`) and a `migrate()` function that reads old state and constructs the new `Eth2Client`. When adding fields to `Eth2Client`, create a new versioned struct matching the *current* on-chain layout and set defaults for new fields in `migrate()`.

## New Relayer (`relayer/`)

Standalone binary with CLI subcommands: `run` (continuous), `run-job` (single execution for Cloud Run), `init`, `generate-config`, `validate-config`. Config via TOML or env vars with `RELAYER_` prefix. Uses Lighthouse types for beacon chain, Alloy for execution RPC, near-fetch for NEAR.
