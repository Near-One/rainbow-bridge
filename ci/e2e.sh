#!/bin/bash
# This test launch all commands and tranfer tokens
# If run locally, you need a manually `node index.js clean`, `npm i -g ganache-cli`
# and also make sure enviroment/vendor/ethashproof exist and build.sh because eth2near-relay
# Always uses that binary instead of ~/.rainbowup

set -exuo pipefail

CI_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
ROOT_DIR=$CI_DIR/..

cd $ROOT_DIR/environment
yarn
node index.js prepare
node index.js start near-node
node index.js start ganache
./scripts/start_ganache.sh &
export GANACHE_PID=$!
trap 'pkill -15 -P $GANACHE_PID' 0
node index.js init-near-contracts
node index.js init-eth-ed25519
node index.js init-near2eth-client
node index.js init-near2eth-prover
node index.js init-eth-erc20
node index.js init-eth-locker
node index.js init-near-fun-token
node index.js start eth-relay
node index.js start near-relay
node index.js transfer-eth-erc20-to-near --amount 1 --eth-sender-sk 0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200 --near-receiver-account alice.test.near --near-master-account nearfuntoken