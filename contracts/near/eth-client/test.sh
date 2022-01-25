#!/usr/bin/env bash

if [ "$1" = "pol" ]; then
    RUST_BACKTRACE=1 cargo test --jobs 8 --package eth-client --no-default-features --features=pol -- --nocapture
else
    RUST_BACKTRACE=1 cargo test --jobs 8 --package eth-client -- --nocapture
fi