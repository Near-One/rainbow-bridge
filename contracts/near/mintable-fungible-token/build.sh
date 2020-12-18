#!/bin/bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

if [[ -z "$BUILDKITE" ]]; then
     userflag="-u $UID:$UID"
else
     userflag=""
fi

docker run \
     --mount type=bind,source=$DIR/..,target=/host \
     --cap-add=SYS_PTRACE --security-opt seccomp=unconfined $userflag \
     -w /host/mintable-fungible-token \
     -e RUSTFLAGS='-C link-arg=-s' \
     nearprotocol/contract-builder \
     cargo +stable build --target wasm32-unknown-unknown --release

cp $DIR/../target/wasm32-unknown-unknown/release/mintable_fungible_token.wasm $DIR/../res/

