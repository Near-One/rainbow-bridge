#!/usr/bin/env bash
set -ex
./index.js clean
./index.js prepare
./index.js start near-node
./index.js start ganache
echo "Sleeping 10 seconds for near node to boot" &&  sleep 10
./index.js init-near-contracts
./index.js init-eth-ed25519
./index.js init-eth-client --eth-client-lock-eth-amount 1000 --eth-client-lock-duration 10
./index.js init-eth-prover
./index.js init-eth-erc20
./index.js init-eth-locker
./index.js init-near-fun-token
./index.js start eth2near-relay
./index.js start near2eth-relay --eth-master-sk 0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501201
./index.js start bridge-watchdog --eth-master-sk 0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501202
./node_modules/.bin/pm2 logs
