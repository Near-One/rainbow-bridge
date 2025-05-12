#!/usr/bin/env bash
rustup target add wasm32-unknown-unknown
cargo near build non-reproducible-wasm --no-abi --no-default-features --features logs
RUST_BACKTRACE=full cargo test --no-default-features --features bls,logs --package eth2-client -- tests::unit_tests::tests::generic_tests::test_gc_headers --nocapture