#!/usr/bin/env bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Exit script as soon as a command fails.
set -o errexit

# Executes cleanup function at script exit.
trap cleanup EXIT

cleanup() {
    # Kill the nearnode instance that we started (if we started one and if it's still running).
    if [ -n "$node_started" ]; then
        docker kill nearcore watchtower
    fi
}

nearnode_port=24567

nearnode_running() {
    nc -z localhost "$nearnode_port"
}

start_nearnode() {
    ./nearcore/scripts/start_localnet.py --home "$DIR/.near"
    nearnode_pid=$!
    sleep 1
}

if nearnode_running; then
    echo "Using existing nearnode instance"
else
    echo "Starting our own nearnode instance"
    rm -rf "$DIR/.near/data"
    start_nearnode
    node_started=1
fi

echo "Creating account for smart contract:"
NODE_ENV=local near --homeDir "$DIR/.near" --keyPath "$DIR/.near/validator_key.json" create_account ethbridge --masterAccount=ethrelay --initialBalance 1000
echo "Deploying smart contract:"
NODE_ENV=local near --homeDir "$DIR/.near" --keyPath "$DIR/.near/validator_key.json" deploy --masterAccount=ethrelay --contractName ethbridge --wasmFile "$DIR/../../ethbridge/res/eth_bridge.wasm"

node "$DIR/../index.js"
