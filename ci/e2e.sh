#!/bin/bash
# This test launch all commands and tranfer tokens
# If run locally, you need a manually `node index.js clean`, `npm i -g ganache-cli`

set -exuo pipefail

CI_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/tmp/ganache.out 2>&1 && pwd )"
ROOT_DIR=$CI_DIR/..

cd $ROOT_DIR/environment
yarn
node index.js clean
node index.js prepare --bridge-src /Users/maksymzavershynskyi/Projects/rainbow-bridge --core-src /Users/maksymzavershynskyi/Projects/nearcore --nearup-src /Users/maksymzavershynskyi/Projects/nearup
node index.js start near-node
node index.js start ganache
./scripts/start_ganache.sh > /dev/null 2>&1 &
export GANACHE_PID=$!
trap 'pkill -15 -P $GANACHE_PID' 0
node index.js init-near-contracts
node index.js init-eth-ed25519
# Use short lockup time for tests
node index.js init-near2eth-client --near2eth-client-lock-eth-amount 1e18 --near2eth-client-lock-duration 10
node index.js init-near2eth-prover
node index.js init-eth-erc20
node index.js init-eth-locker
node index.js init-near-fun-token
# First start pm2 daemon
pm2 ping
sleep 5
pm2 list
node index.js start near-relay --eth-master-sk 0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501201
sleep 5
pm2 list
node index.js start eth-relay
sleep 5
pm2 list
node index.js transfer-eth-erc20-to-near --amount 1000 \
--eth-sender-sk 0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200 \
--near-receiver-account eth2nearprover --near-master-account nearfuntoken \
2>&1 | tee -a /tmp/eth2neartransfer.out
grep "Balance of eth2nearprover after the transfer is 1000" /tmp/eth2neartransfer.out
node index.js transfer-eth-erc20-from-near --amount 1 --near-sender-account eth2nearprover \
--near-sender-sk ed25519:3D4YudUQRE39Lc4JHghuB5WM8kbgDDa34mnrEP5DdTApVH81af7e2dWgNPEaiQfdJnZq1CNPp5im4Rg5b733oiMP \
--eth-receiver-address 0xEC8bE1A5630364292E56D01129E8ee8A9578d7D8 \
2>&1 | tee -a /tmp/near2ethtransfer.out
grep "after the transfer: 1" /tmp/near2ethtransfer.out
