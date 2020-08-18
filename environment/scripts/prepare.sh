#!/bin/bash
set -euo pipefail

eval RAINBOW_DIR=~/.rainbow

export LOCAL_BRIDGE_SRC
export LOCAL_CORE_SRC
export LOCAL_NEARUP_SRC

eval BRIDGE_SRC=~/.rainbow/bridge
eval CORE_SRC=~/.rainbow/core
eval NEARUP_SRC=~/.rainbow/nearup
eval NEARUP_LOGS=~/.nearup/localnet-logs

mkdir -p $RAINBOW_DIR
mkdir -p $RAINBOW_DIR/logs/ganache
mkdir -p $RAINBOW_DIR/logs/near-relay
mkdir -p $RAINBOW_DIR/logs/eth-relay
mkdir -p $RAINBOW_DIR/logs/near-watchdog
touch $RAINBOW_DIR/logs/ganache/out.log
touch $RAINBOW_DIR/logs/ganache/err.log
touch $RAINBOW_DIR/logs/near-relay/out.log
touch $RAINBOW_DIR/logs/near-relay/err.log
touch $RAINBOW_DIR/logs/eth-relay/out.log
touch $RAINBOW_DIR/logs/eth-relay/err.log
touch $RAINBOW_DIR/logs/near-watchdog/out.log
touch $RAINBOW_DIR/logs/near-watchdog/err.log

if test -z "$LOCAL_CORE_SRC"
then
echo "near-core home not specified..."
git clone "https://github.com/nearprotocol/nearcore" $CORE_SRC
eval CURR_DIR=$(pwd)
cd $CURR_DIR
else
echo "Linking the specified local repo from ${LOCAL_CORE_SRC} to ${CORE_SRC}"
ln -s $LOCAL_CORE_SRC $CORE_SRC
fi

if test -z "$LOCAL_BRIDGE_SRC"
then
echo "rainbow-bridge home not specified..."
git clone "https://github.com/near/rainbow-bridge/" $BRIDGE_SRC
else
echo "Linking the specified local repo from ${LOCAL_BRIDGE_SRC} to ${BRIDGE_SRC}"
ln -s $LOCAL_BRIDGE_SRC $BRIDGE_SRC
fi

if test -z "$LOCAL_NEARUP_SRC"
then
echo "nearup home not specified..."
git clone "https://github.com/near/nearup/" $NEARUP_SRC
else
echo "Linking the specified local repo from ${LOCAL_NEARUP_SRC} to ${NEARUP_SRC}"
ln -s $LOCAL_NEARUP_SRC $NEARUP_SRC
fi
mkdir -p $NEARUP_LOGS

cd $BRIDGE_SRC
git submodule update --init --recursive

cd $CORE_SRC
cargo build --package neard --bin neard
echo "Compiled source of nearcore"

cd $BRIDGE_SRC/libs-rs
./build_all.sh
echo "Compiled Rust contracts"

cd $BRIDGE_SRC/libs-sol
./build_all.sh
echo "Built Solidity contracts"

# Install environment dependencies
cd $BRIDGE_SRC/environment
yarn

cd $BRIDGE_SRC/environment/vendor/ganache
yarn

cd $BRIDGE_SRC/environment/vendor/ethashproof
./build.sh
echo 'Compiled ethashproof module'

# Start the pm2 daemon if it is currently not running.
yarn pm2 ping
