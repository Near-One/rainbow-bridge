#!/usr/bin/env bash

rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release --no-default-features --features logs

RUST_BACKTRACE=1 cargo test --jobs 8 --package eth2-client -- --nocapture

RUST_BACKTRACE=1 cargo test --no-default-features --jobs 8 --package eth2-client -- --nocapture

RUST_BACKTRACE=1 cargo test --no-default-features --features bls --jobs 8 --package eth2-client -- --nocapture
