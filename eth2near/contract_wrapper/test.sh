#!/usr/bin/env bash

cd ../../contracts/near/eth2-client
cargo build --target wasm32-unknown-unknown --release --no-default-features --features logs
cd -

RUST_BACKTRACE=1 cargo test --jobs 8 -- --nocapture
