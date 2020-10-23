 #!/bin/bash
set -euo pipefail

CI_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
ROOT_DIR=$CI_DIR/..

mkdir -p $ROOT_DIR/testdata
cd $ROOT_DIR/testdata
curl https://s3-us-west-1.amazonaws.com/rainbow-bridge.nearprotocol.com/test-data/near-proofs.tar.gz -o near-proofs.tar.gz
tar zxf near-proofs.tar.gz

cd $ROOT_DIR
yarn

cd $ROOT_DIR/nearprover
yarn
NEAR_PROOFS_DIR=$ROOT_DIR/testdata/near-proofs yarn test
