 #!/bin/bash
set -euo pipefail

CI_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
ROOT_DIR=$CI_DIR/..

cd $ROOT_DIR
yarn

cd $ROOT_DIR/nearprover
yarn
NEAR_PROOFS_DIR=$ROOT_DIR/testdata/near-proofs yarn test
