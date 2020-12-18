#!/bin/bash
set -euo pipefail

CI_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
ROOT_DIR=$CI_DIR/..

mkdir -p $ROOT_DIR/testdata
cd $ROOT_DIR/testdata
curl https://s3-us-west-1.amazonaws.com/rainbow-bridge.nearprotocol.com/test-data/eth-headers.tar.gz -o eth-headers.tar.gz
tar zxf eth-headers.tar.gz

cd $ROOT_DIR/eth-client
ETH_HEADER_DIR=$ROOT_DIR/testdata/headers cargo test --package eth-client --features expensive_tests --lib -- tests::predumped_block_can_be_added --exact --nocapture
