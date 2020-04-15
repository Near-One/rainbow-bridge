#!/usr/bin/env bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Exit script as soon as a command fails.
set -o errexit

# Executes cleanup function at script exit.
trap cleanup EXIT

waitport() {
    while ! nc -z localhost $1 ; do sleep 1 ; done
}

cleanup() {
    # Kill the nearnode instance that we started (if we started one and if it's still running).
    if [ -n "$node_started" ]; then
        docker kill nearcore watchtower > /dev/null &
    fi
}

nearnode_port=24567

nearnode_running() {
    nc -z localhost "$nearnode_port"
}

start_nearnode() {
    echo "ethrelay" | "$DIR/start_localnet.py" --home "$DIR/.near" --image "nearprotocol/nearcore:ethdenver"
    waitport $nearnode_port
}

if nearnode_running; then
    echo "Using existing nearnode instance"
else
    echo "Starting our own nearnode instance"
    rm -rf "$DIR/.near"
    start_nearnode
    node_started=1
fi

NODE_URL="http://localhost:3030"

# The master account ID to create accounts
NEAR_MASTER_ACCOUNT_ID="ethrelay"
# The account ID to push blocks
NEAR_RELAYER_ACCOUNT_ID="ethrelay-pusher"
# The account ID used as ETH bridge
NEAR_ETHBRIDGE_ACCOUNT_ID="ethbridge"

echo "Creating account to push blocks:"
NODE_ENV=local yarn run near --nodeUrl=$NODE_URL --homeDir "$DIR/.near" --keyPath "$DIR/.near/validator_key.json" create_account $NEAR_RELAYER_ACCOUNT_ID --masterAccount=$NEAR_MASTER_ACCOUNT_ID --initialBalance 100000000 || echo "Skip creating ethbridge accout"
echo "Creating account for smart contract:"
NODE_ENV=local yarn run near --nodeUrl=$NODE_URL --homeDir "$DIR/.near" --keyPath "$DIR/.near/validator_key.json" create_account $NEAR_ETHBRIDGE_ACCOUNT_ID --masterAccount=$NEAR_MASTER_ACCOUNT_ID --initialBalance 100000000 || echo "Skip creating ethbridge accout"
echo "Deploying smart contract:"
NODE_ENV=local yarn run near --nodeUrl=$NODE_URL --homeDir "$DIR/.near" --keyPath "$DIR/.near/validator_key.json" deploy --contractName $NEAR_ETHBRIDGE_ACCOUNT_ID --wasmFile "$DIR/../../ethbridge/res/eth_bridge.wasm" || echo "Skip deploying ethbridge smart contract"

BRIDGE_VALIDATE_ETHASH=true \
    NEAR_NODE_URL="http://localhost:3030" \
    NEAR_NODE_NETWORK_ID=local \
    NEAR_RELAYER_ACCOUNT_ID=$NEAR_RELAYER_ACCOUNT_ID \
    NEAR_ETHBRIDGE_ACCOUNT_ID=$NEAR_ETHBRIDGE_ACCOUNT_ID \
    ETHEREUM_NODE_URL="wss://mainnet.infura.io/ws/v3/b5f870422ee5454fb11937e947154cd2" \
    node "$DIR/../index.js"
