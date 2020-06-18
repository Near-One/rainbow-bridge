#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -e

# Build Near2EthClient contract
echo "Building Near2EthClient and ED25519 contracts"
pushd "nearbridge"
yarn
./dist.sh
popd

# Build emitter contract
echo "Building emitter"
pushd "emitter"
yarn
yarn run oz compile --no-interactive
popd

echo "Building token locker"
# Build contracts for locking token.
pushd "token-locker"
yarn
./dist.sh
popd

