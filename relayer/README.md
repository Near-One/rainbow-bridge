# Ethereum to NEAR Light Client Relayer

A Rust-based service that relays Ethereum beacon chain and execution layer data to NEAR smart contracts, enabling trustless verification of Ethereum state on NEAR.

## Overview

The relayer bridges Ethereum consensus and execution data to NEAR by operating in two modes:

- **SubmitLightClientUpdate**: Relays Ethereum beacon chain finality proofs (light client updates) to keep NEAR's view of Ethereum consensus synchronized
- **SubmitHeader**: Relays Ethereum execution layer block headers to NEAR for recent block verification

## Architecture

### Core Components

- **EthRelayer** (`src/relay.rs`): Main orchestrator managing the relay loop
- **BeaconClient** (`src/clients/beacon.rs`): Interfaces with Ethereum beacon chain API  
- **ExecutionClient** (`src/clients/execution.rs`): Fetches Ethereum execution layer blocks
- **ContractClient** (`src/clients/near.rs`): Submits data to NEAR smart contracts

### Data Flow

1. Check if beacon node is synced
2. Query NEAR contract for current client mode
3. Fetch and submit either light client updates or execution headers
4. Sleep and repeat

## Quick Start

```bash
# Generate example configuration
cargo run -- generate-config

# Validate configuration
cargo run -- validate-config

# Run continuously (automatically uses relayer.toml if present)
cargo run -- run

# Run single job (for Cloud Run Jobs)
cargo run -- run-job
```

## Configuration

Configuration sources (in priority order):
1. Environment variables (prefixed with `RELAYER_`)
2. Config file (automatically uses `relayer.toml` if present, or specify with `--config`)  
3. Default values

### Key Settings

```toml
[beacon]
endpoint = "http://beacon-node.example.com"
timeout_secs = 30

[execution]
endpoint = "https://eth-rpc.example.com"
max_batch_size = 1000

[near]
endpoint = "https://rpc.mainnet.near.org"
contract_account_id = "eth-client.near"
signer_account_id = "relayer.near"
secret_key = "ed25519:..."

[relayer]
update_interval_epochs = 1
headers_batch_size = 100
dry_run = false
```

### Environment Variables

Use double underscores for nested sections:

```bash
export RELAYER_NEAR__CONTRACT_ACCOUNT_ID="eth-client.near"
export RELAYER_NEAR__SECRET_KEY="ed25519:..."
export RELAYER_RELAYER__DRY_RUN=false
```

## Deployment

### Docker

Build from project root (requires local dependencies):

```bash
docker build -f relayer/Dockerfile -t eth-relayer .
docker run --env-file .env eth-relayer
```

### Cloud Run Jobs

Use `run-job` command for single execution:
- Executes once and exits with appropriate status code
- Configure via environment variables
- Suitable for scheduled execution

## Development

```bash
# Build
cargo build

# Test
cargo test

# Run with custom config
cargo run -- run --config custom.toml

# Dry run mode
RELAYER_RELAYER__DRY_RUN=true cargo run -- run
```

## Dependencies

- **Lighthouse**: Ethereum beacon chain types and utilities
- **Alloy**: Ethereum execution layer RPC
- **NEAR SDK**: Blockchain interactions
- **Local**: `eth-types`, `eth2-utility` from `../contracts/near/`