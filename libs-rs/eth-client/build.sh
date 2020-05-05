#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -e

RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/eth_client.wasm ./res/
