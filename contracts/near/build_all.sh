#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

cd $DIR/eth-client
./build.sh
cd $DIR/eth-prover
./build.sh
cd $DIR/mintable-fungible-token
./build.sh
