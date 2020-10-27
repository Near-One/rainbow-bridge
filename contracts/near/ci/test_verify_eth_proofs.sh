 #!/bin/bash
set -euo pipefail

CI_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
ROOT_DIR=$CI_DIR/..


mkdir -p $ROOT_DIR/testdata
cd $ROOT_DIR/testdata
curl https://s3-us-west-1.amazonaws.com/rainbow-bridge.nearprotocol.com/test-data/eth-proofs.tar.gz -o eth-proofs.tar.gz
tar zxf eth-proofs.tar.gz

cd $ROOT_DIR/eth-prover
ETH_PROOF_DIR=$ROOT_DIR/testdata/eth-proofs cargo test --package eth-prover --features expensive_tests --lib -- tests::verify_dumped_log_entries --exact --nocapture