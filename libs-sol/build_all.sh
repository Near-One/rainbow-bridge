#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -e

# Build NearOnEthClient contract
echo "Building NearOnEthClient and ED25519 contracts"
pushd "nearbridge"
yarn
./dist.sh
popd

echo "Building NearOnEthProver contract"
pushd "nearprover"
yarn
./dist.sh
popd

echo "Building token locker"
# Build contracts for locking token.
pushd "token-locker"
yarn
./dist.sh
popd

