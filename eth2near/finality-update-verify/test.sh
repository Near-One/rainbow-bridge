#!/usr/bin/env bash

cargo build
RUST_BACKTRACE=1 cargo test --jobs 8 -- --nocapture
