#!/usr/bin/env bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Exit script as soon as a command fails.
set -o errexit

# Executes cleanup function at script exit.
trap cleanup EXIT

cleanup() {
    # Kill the nearnode instance that we started (if we started one and if it's still running).
    if [ -n "$nearcode_started" ]; then
        docker kill nearcore watchtower > /dev/null &
    fi
    if [ -n "$ganache_started" ]; then
        kill $ganache_pid > /dev/null &
    fi
}

nearnode_port=24567

nearnode_running() {
    nc -z localhost "$nearnode_port"
}

start_nearnode() {
    echo "ethrelay" | "$DIR/start_localnet.py" --home "$DIR/.near" --image "nearprotocol/nearcore:nofloatsfixedgas"
    trap "docker kill nearcore watchtower > /dev/null &" EXIT INT TERM
    nearcode_started=1
    sleep 1
}

if nearnode_running; then
    echo "Using existing nearnode instance"
else
    echo "Starting our own nearnode instance"
    rm -rf "$DIR/.near"
    start_nearnode
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

    ganache-cli --fork "https://mainnet.infura.io/v3/b5f870422ee5454fb11937e947154cd2" --gasLimit 10000000 -p "$ganache_port" "${accounts[@]}" > /dev/null &
    ganache_pid=$!
    trap "kill $ganache_pid" EXIT INT TERM
    ganache_started=1
    echo "ganache_pid: $ganache_pid"
}

if ganache_running; then
    echo "Using existing ganache instance"
else
    echo "Starting our own ganache instance"
    start_ganache
fi

echo "Creating account for smart contract:"
NODE_ENV=local yarn run near --homeDir "$DIR/.near" --keyPath "$DIR/.near/validator_key.json" create_account ethbridge --masterAccount=ethrelay --initialBalance 100000000 || echo "Skip creating ethbridge accout"
#echo "Deploying smart contract:"
#NODE_ENV=local yarn run near --homeDir "$DIR/.near" --keyPath "$DIR/.near/validator_key.json" deploy --masterAccount=ethrelay --contractName ethbridge --wasmFile "$DIR/../../ethbridge/res/eth_bridge.wasm" || echo "Skip deploying ethbridge smart contract"

NEAR_BRIDGE_OWNER_PRIVATE_KEY=0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200 node "$DIR/../index.js"
