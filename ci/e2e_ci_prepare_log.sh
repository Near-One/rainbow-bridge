#!/bin/bash
# hard link pm2 logs to current dir, so buildkite can pick up them as artifacts

mkdir -p ~/.rainbow/logs/eth-relay
mkdir -p ~/.rainbow/logs/near-relay
mkdir -p ~/.rainbow/logs/ganache
mkdir -p ~/.rainbow/logs/near-watchdog
touch eth-relay-out.log
touch eth-relay-err.log
touch near-relay-out.log
touch near-relay-err.log
touch ganache-out.log
touch ganache-err.log
touch near-watchdog-out.log
touch near-watchdog-err.log
ln eth-relay-out.log ~/.rainbow/logs/eth-relay/out.log || true
ln eth-relay-err.log ~/.rainbow/logs/eth-relay/err.log || true
ln near-relay-out.log ~/.rainbow/logs/near-relay/out.log || true
ln near-relay-err.log ~/.rainbow/logs/near-relay/err.log || true
ln ganache-out.log ~/.rainbow/logs/ganache/out.log || true
ln ganache-err.log ~/.rainbow/logs/ganache/err.log || true
ln near-watchdog-out.log ~/.rainbow/logs/near-watchdog/out.log || true
ln near-watchdog-err.log ~/.rainbow/logs/near-watchdog/err.log || true
