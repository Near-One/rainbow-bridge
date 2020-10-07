#!/bin/bash

# Exit script as soon as a command fails.
set -o errexit

TEST_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/tmp/ganache.out 2>&1 && pwd )"
ROOT_DIR=$TEST_DIR/..

# Executes cleanup function at script exit.
trap cleanup EXIT
cleanup() {
    cd $ROOT_DIR
    #node index.js stop all
}

cd $ROOT_DIR
yarn

node index.js clean
if [ -n "${LOCAL_CORE_SRC+x}" ]; then
  node index.js prepare --core-src "$LOCAL_CORE_SRC"
else
  node index.js prepare
fi
node index.js start near-node
node index.js start ganache
# Wait for the local node to start
while ! curl localhost:3030; do
  sleep 1
done

while ! curl localhost:9545; do
  sleep 1
done

echo "


=== Running test $1 ===
"

cd $TEST_DIR

node $1
