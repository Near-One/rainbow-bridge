#!/usr/bin/env bash

RUST_BACKTRACE=1 cargo test bsc_update_epoch_header --package eth-client --features=bsc -- --nocapture
# RUST_BACKTRACE=1 cargo test --jobs 8 --package eth-client --features=bsc -- --nocapture
