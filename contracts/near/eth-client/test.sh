#!/usr/bin/env bash

if [ "$1" = "bsc" ]; then
    RUST_BACKTRACE=1 cargo test bsc_validate_header_12058600 --package eth-client --no-default-features --features=bsc -- --nocapture
else
    RUST_BACKTRACE=1 cargo test --jobs 8 --package eth-client --features=bsc -- --nocapture
fi
