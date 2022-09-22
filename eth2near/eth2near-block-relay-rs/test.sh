#!/usr/bin/env bash

git secret reveal -f

cargo build

RUST_BACKTRACE=1 cargo test --jobs 8 -- --nocapture
