#!/usr/bin/env bash
rustup target add wasm32-unknown-unknown
cargo near build non-reproducible-wasm

cp ../target/near/eth2_client/eth2_client.wasm ../res/
