#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -e

# Build emitter contract
pushd emitter
yarn
yarn run oz compile
popd

