#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -ex

echo "Building NearOnEthClient and ED25519 contracts"
cd nearbridge
yarn
./dist.sh

echo "Building NearOnEthProver contract"
cd ../nearprover
yarn
./dist.sh

echo "Building token locker"
cd ../token-locker
yarn
./dist.sh
