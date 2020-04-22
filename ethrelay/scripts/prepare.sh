#!/usr/bin/env bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

rm -rf "$DIR/.near"
cargo run --package neard --bin neard  -- --home "$DIR/.near" init --chain-id= --test-seed=ethrelay --account-id=ethrelay --fast
