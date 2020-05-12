#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -e

./eth-client/build.sh
./eth-prover/build.sh