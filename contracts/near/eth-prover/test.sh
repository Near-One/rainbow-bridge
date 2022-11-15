#!/usr/bin/env bash

cargo test --no-default-features --jobs 8 --package eth-prover
cargo test --jobs 8 --package eth-prover