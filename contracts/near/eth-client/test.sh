#!/usr/bin/env bash

# Run without default features
RUST_BACKTRACE=1 cargo test --no-default-features --jobs 8 --package eth-client -- --nocapture
# Run with default features
RUST_BACKTRACE=1 cargo test --jobs 8 --package eth-client -- --nocapture
