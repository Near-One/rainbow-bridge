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

## Test, Lint

All contract commands run from `contracts/near/`:

```bash
# Test (the canonical CI sequence — builds test WASM first, then runs tests twice: with and without default features)
cd contracts/near/eth2-client && ./test.sh

# Run all tests
cd contracts/near && cargo test -p eth2-client

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

The eth2-client alternates between two modes (`ClientMode` enum):

```
  ┌───────────────────────────────────────────────────────────┐
  │                                                           │
  ▼                                                           │
SubmitLightClientUpdate                                       │
  │  submit_beacon_chain_light_client_update(update)          │
  │    1. verify_finality_branch: merkle proof that           │
  │       update.finalized_header is in attested_header       │
  │    2. check ≥2/3 sync committee participation             │
  │    3. verify_bls_signatures (aggregate BLS via            │
  │       NEAR host fns: bls12381_p1_sum, pairing_check)      │
  │    4. commit: if period advanced (P→P+1), rotate          │
  │       current_sync_committee ← next, next ← from update   │
  │    5. set finalized_beacon_header, switch mode            │
  │                                                           │
  ▼                                                           │
SubmitHeader                                                  │
  │  submit_execution_header(block_header) ×N                 │
  │    — called once per block, newest→oldest                 │
  │    — first call: block whose hash matches                 │
  │      finalized_beacon_header.execution_block_hash         │
  │    — each subsequent: parent of the previous              │
  │    — inserts block_number→block_hash in LookupMap         │
  │    — GCs blocks beyond hashes_gc_threshold                │
  │    — last call: block_number == old_finalized + 1,        │
  │      verifies parent_hash links to old finalized hash ────┘
  │      (chain is "closed"), updates finalized_execution_header
```

**Period constraint**: light client updates can advance by at most 1 sync committee period (`update_period == finalized_period || finalized_period + 1`). This bounds the execution block gap (and thus GC work) to at most one period's worth of blocks per cycle.

**GC**: `gc_finalized_execution_blocks` removes old entries from `finalized_execution_blocks` LookupMap. Bounded to `MAX_GC_BLOCKS_PER_CALL` (= one period = 8192) removals per call. In steady state this removes exactly 1 block per `submit_execution_header` call.

## Key Protocol Constants

Defined in `eth2-utility/src/consensus.rs`, per the [Ethereum Consensus Spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md):

- `SLOTS_PER_EPOCH = 32` (~6.4 min)
- `EPOCHS_PER_SYNC_COMMITTEE_PERIOD = 256`
- One sync committee period = 32 × 256 = 8192 slots ≈ 27 hours
- `MIN_SYNC_COMMITTEE_PARTICIPANTS = 1` (contract enforces ≥2/3 of 512)
- `hashes_gc_threshold` is typically 51000 (~7 days of blocks)

## Test Data

Test data lives in `contracts/near/eth2-client/src/data/` (LFS-tracked JSON files). Sepolia is the primary test network. Regenerate with:
```bash
cd contracts/near/eth2-client/src/data && uv run dump_sepolia_data.py
```

## New Relayer (`relayer/`)

Standalone binary with CLI subcommands: `run` (continuous), `run-job` (single execution for Cloud Run), `init`, `generate-config`, `validate-config`. Config via TOML or env vars with `RELAYER_` prefix. Uses Lighthouse types for beacon chain, Alloy for execution RPC, near-fetch for NEAR.
