#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -e

RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/eth_prover.wasm ./res/
#wasm-snip res/eth_prover.wasm -o res/eth_prover.wasm
#wasm-opt -Oz --output ./res/eth_prover.wasm ./res/eth_prover.wasm
#rm -rf target
