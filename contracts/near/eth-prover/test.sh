#!/usr/bin/env bash

RUST_BACKTRACE=1 cargo test --no-default-features --jobs 8 --package eth-prover -- --nocapture
RUST_BACKTRACE=1 cargo test --jobs 8 --package eth-prover -- --nocapture
