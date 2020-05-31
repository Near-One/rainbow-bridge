#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -e

# Build emitter contract
echo "Building emitter"
pushd "emitter"
yarn
yarn run oz compile
popd

echo "Building token locker"
# Build contracts for locking token.
pushd "example-token-locker"
yarn
./dist.sh
popd

