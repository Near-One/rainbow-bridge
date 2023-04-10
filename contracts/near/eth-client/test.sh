#!/usr/bin/env bash

# Run with default features
RUST_BACKTRACE=1 cargo test --jobs 1 --package eth-client -- --test-threads 1 --nocapture 
