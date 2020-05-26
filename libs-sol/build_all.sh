#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -e

# Build emitter contract
for CONTRACT in emitter example-token-locker
do
  pushd "$CONTRACT"
  yarn
  yarn run oz compile
  popd
done

