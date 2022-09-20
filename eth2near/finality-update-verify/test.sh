#!/usr/bin/env bash

git lfs fetch --all
cargo build
RUST_BACKTRACE=1 cargo test --jobs 8 -- --nocapture
