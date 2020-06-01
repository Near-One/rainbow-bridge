#!/bin/bash
set -euo pipefail

RAINBOWUP_DIR=~/.rainbowup

export BRIDGE_SRC
export CORE_SRC

mkdir -p $RAINBOWUP_DIR

if [ ! -d "$CORE_SRC" ]
then
echo "near-core repo not specified..."
git clone "https://github.com/nearprotocol/nearcore" $CORE_SRC
fi

if [ ! -d "$BRIDGE_SRC" ]
then
echo "rainbow-bridge repo not specified..."
git clone "https://github.com/near/rainbow-bridge/" $BRIDGE_SRC
fi

cd $BRIDGE_SRC
git submodule update --init --recursive

cd $CORE_SRC
cargo build --package neard --bin neard
echo "Compiled source of nearcore"

cd $BRIDGE_SRC/libs-rs
./build_all.sh
echo "Compiled Rust contracts"

cd $BRIDGE_SRC/libs-sol
yarn
./build_all.sh
echo "Built Solidity contracts"

# Install environment dependencies
cd $BRIDGE_SRC/environment
yarn

cd $BRIDGE_SRC/environment/vendor/ethashproof
./build.sh
echo 'Compiled ethashproof module'
