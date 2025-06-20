# Test Data Generator

Generates test data for ETH light client integration tests on NEAR.

## Usage

### Generate Sepolia Test Data
```bash
cd contracts/near/eth2-client/src/data/
uv run dump_sepolia_data.py
```

### Dependencies
Uses `uv` for fast dependency management:
```bash
# Install uv if needed
curl -LsSf https://astral.sh/uv/install.sh | sh

# Dependencies auto-installed on first run with uv
# Or manually: uv add requests web3 tqdm eth2spec
```

### Environment Variables
- `NETWORK`: Network name (default: `sepolia`)
- `EXECUTION_RPC`: Ethereum RPC endpoint
- `CONSENSUS_API`: Beacon chain API endpoint
- `BLOCK_WINDOW`: Block window size (default: `50`)
- `BATCH_SIZE`: RPC batch size (default: `50`)

### Output Files
- `{network}/execution_blocks_{start}_{end}.json`: Execution block headers
- `{network}/light_client_update_period_{period}.json`: Light client updates
- `{network}/beacon_header_{slot}.json`: Beacon block headers

## Integration Tests

The script generates data used by:
- `get_sepolia_test_data()` in `tests/utils.rs`
- Integration tests in `tests/integration_tests.rs`

### Update Test Data
1. Run the script to generate new data files
2. Update hardcoded block numbers and periods in `tests/utils.rs:106-122`
3. Run tests: `cargo test`

### How It Works
- Fetches 4 most recent light client update periods
- Downloads corresponding execution blocks in 2 batches
- Generates Merkle proofs for block hash verification
- Creates normalized JSON files matching Rust struct format

The test expects specific file patterns matching the periods and block ranges defined in `utils.rs`.