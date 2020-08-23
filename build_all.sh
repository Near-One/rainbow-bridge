#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -e

echo "Building NearOnEthClient and ED25519 contracts"
./dist.sh nearbridge

echo "Building NearOnEthProver contract"
./dist.sh nearprover

echo "Building token locker"
./dist.sh token-locker
