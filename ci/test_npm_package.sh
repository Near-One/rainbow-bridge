#!/bin/bash
# This test install current package as if current package is published to npm
# And verify everything of the npm package is good. It should pass before publish
# npm package

set -exuo pipefail

CI_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/tmp/ganache.out 2>&1 && pwd )"
ROOT_DIR=$CI_DIR/..

cd ${ROOT_DIR}
rm -rf testenv
mkdir testenv
cd testenv
npm init -y > /dev/null
npm i ${ROOT_DIR}/environment
npm i pm2
export PATH=${ROOT_DIR}/testenv/node_modules/.bin:$PATH
cd ..

rainbow clean
if [ -n "${LOCAL_CORE_SRC+x}" ]; then
  rainbow prepare --core-src "$LOCAL_CORE_SRC"
else
  rainbow prepare
fi
rainbow start near-node
rainbow start ganache
# Wait for the local node to start
while ! curl localhost:3030; do
  sleep 1
done

while ! curl localhost:9545; do
  sleep 1
done

rainbow init-near-contracts
rainbow init-eth-ed25519
# Use short lockup time for tests
rainbow init-eth-client --eth-client-lock-eth-amount 1e18 --eth-client-lock-duration 30
rainbow init-eth-prover
rainbow init-eth-erc20
rainbow init-eth-locker
rainbow init-near-fun-token
# First start pm2 daemon
cd ${ROOT_DIR}/testenv/
pm2 ping
sleep 5
pm2 list
rainbow start near-relay --eth-master-sk 0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501201
sleep 5
pm2 list
rainbow start eth-relay
sleep 5
pm2 list
rainbow transfer-eth-erc20-to-near --amount 1000 \
--eth-sender-sk 0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200 \
--near-receiver-account rainbow_bridge_eth_on_near_prover --near-master-account rainbow_bridge_eth_on_near_prover \
2>&1 | tee -a /tmp/eth2neartransfer.out
grep "Balance of rainbow_bridge_eth_on_near_prover after the transfer is 1000" /tmp/eth2neartransfer.out
rainbow transfer-eth-erc20-from-near --amount 1 --near-sender-account rainbow_bridge_eth_on_near_prover \
--near-sender-sk ed25519:3D4YudUQRE39Lc4JHghuB5WM8kbgDDa34mnrEP5DdTApVH81af7e2dWgNPEaiQfdJnZq1CNPp5im4Rg5b733oiMP \
--eth-receiver-address 0xEC8bE1A5630364292E56D01129E8ee8A9578d7D8 \
2>&1 | tee -a /tmp/near2ethtransfer.out
grep "after the transfer: 1" /tmp/near2ethtransfer.out
