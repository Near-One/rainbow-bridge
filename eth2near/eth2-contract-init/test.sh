#!/usr/bin/env bash

# for running tests localy: 
# export ETH1_INFURA_API_KEY=<YOUR_INFURA_API_KEY>

cargo build

RUST_BACKTRACE=1 cargo test --jobs 8 -- --nocapture
