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

    # Kill the ganache instance that we started (if we started one and if it's still running).
    if [ -n "$ganache_pid" ] && ps -p $ganache_pid > /dev/null; then
        kill $ganache_pid
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

ganache_port=9545

ganache_running() {
    nc -z localhost "$ganache_port"
}

start_ganache() {
    local accounts=(
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501201,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501202,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501203,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501204,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501205,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501206,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501207,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501208,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501209,1000000000000000000000000"
    )

    yarn run ganache-cli --blockTime 12 --gasLimit 10000000 -p "$ganache_port" "${accounts[@]}" > /dev/null &
    ganache_pid=$!
    waitport $ganache_port
}

if ganache_running; then
    echo "Using existing ganache instance"
else
    echo "Starting our own ganache instance"
    start_ganache
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

BRIDGE_VALIDATE_ETHASH=false \
    NEAR_NODE_URL="http://localhost:3030" \
    NEAR_NODE_NETWORK_ID=local \
    NEAR_RELAYER_ACCOUNT_ID=$NEAR_RELAYER_ACCOUNT_ID \
    NEAR_ETHBRIDGE_ACCOUNT_ID=$NEAR_ETHBRIDGE_ACCOUNT_ID \
    ETHEREUM_NODE_URL="ws://localhost:$ganache_port" \
    node "$DIR/../index.js" &

# Successfully stop after 5m
sleep 300 && kill -0 $$ && echo "Successfully worked for 5m, stopped"
