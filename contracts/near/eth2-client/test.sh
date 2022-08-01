#!/usr/bin/env bash

RUST_BACKTRACE=1 cargo test --jobs 8 --package eth2-client -- --nocapture

RUST_BACKTRACE=1 cargo test --features bls --jobs 8 --package eth2-client -- --nocapture
