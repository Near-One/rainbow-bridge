#!/usr/bin/env bash

if [ "$1" = "bsc" ]; then
    RUST_BACKTRACE=1 cargo test --jobs 8 --package eth-prover --no-default-features -- --nocapture
else
    RUST_BACKTRACE=1 cargo test --jobs 8 --package eth-prover -- --nocapture
fi
