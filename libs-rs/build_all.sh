#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

$DIR/eth-client/build.sh
$DIR/eth-prover/build.sh
$DIR/fungible-token/build.sh
