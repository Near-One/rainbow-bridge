#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -e

RUSTFLAGS='-C link-arg=-s' cargo +nightly build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/eth_bridge.wasm ./res/
#wasm-opt -Oz --output ./res/eth_bridge.wasm ./res/eth_bridge.wasm
#rm -rf target
