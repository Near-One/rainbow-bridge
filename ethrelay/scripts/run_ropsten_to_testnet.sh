#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -o errexit

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
NODE_URL="https://rpc.nearprotocol.com"

echo "Creating account for smart contract:"
NODE_ENV=local yarn run near --nodeUrl=$NODE_URL --homeDir "$DIR/.near" --keyPath "$DIR/.near/validator_key.json" create_account ethbridge --masterAccount=ethrelay --initialBalance 100000000 || echo "Skip creating ethbridge accout"
echo "Deploying smart contract:"
NODE_ENV=local yarn run near --nodeUrl=$NODE_URL --homeDir "$DIR/.near" --keyPath "$DIR/.near/validator_key.json" deploy --contractName ethbridge --wasmFile "$DIR/../../ethbridge/res/eth_bridge.wasm" || echo "Skip deploying ethbridge smart contract"

# Launch EthRelay
BRIDGE_VALIDATE_ETHASH=true \
    NEAR_NODE_URL="https://rpc.nearprotocol.com" \
    NEAR_NODE_NETWORK_ID=local \
    ETHEREUM_NODE_URL="wss://ropsten.infura.io/ws/v3/b5f870422ee5454fb11937e947154cd2" \
    node "$DIR/../index.js"
