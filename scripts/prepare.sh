#!/bin/bash
set -euo pipefail

eval RAINBOW_DIR=~/.rainbow

export LOCAL_CORE_SRC
export LOCAL_NEARUP_SRC

eval CORE_SRC=~/.rainbow/core
SCRIPTS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" 2>&1 && pwd )"
eval BRIDGE_SRC=${SCRIPTS_DIR}/..
eval LIBS_SOL_SRC=${BRIDGE_SRC}/node_modules/rainbow-bridge-sol
eval LIBS_RS_SRC=${BRIDGE_SRC}/node_modules/rainbow-bridge-rs
eval NEARUP_SRC=~/.rainbow/nearup
eval NEARUP_LOGS=~/.nearup/localnet-logs

mkdir -p $RAINBOW_DIR
mkdir -p $RAINBOW_DIR/logs/ganache
mkdir -p $RAINBOW_DIR/logs/near2eth-relay
mkdir -p $RAINBOW_DIR/logs/eth2near-relay
mkdir -p $RAINBOW_DIR/logs/watchdog
touch $RAINBOW_DIR/logs/ganache/out.log
touch $RAINBOW_DIR/logs/ganache/err.log
touch $RAINBOW_DIR/logs/near2eth-relay/out.log
touch $RAINBOW_DIR/logs/near2eth-relay/err.log
touch $RAINBOW_DIR/logs/eth2near-relay/out.log
touch $RAINBOW_DIR/logs/eth2near-relay/err.log
touch $RAINBOW_DIR/logs/watchdog/out.log
touch $RAINBOW_DIR/logs/watchdog/err.log

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

if test -z "$LOCAL_NEARUP_SRC"
then
echo "nearup home not specified..."
git clone "https://github.com/near/nearup/" $NEARUP_SRC
else
echo "Linking the specified local repo from ${LOCAL_NEARUP_SRC} to ${NEARUP_SRC}"
ln -s $LOCAL_NEARUP_SRC $NEARUP_SRC
fi
mkdir -p $NEARUP_LOGS

cd $CORE_SRC
cargo build --package neard --bin neard
echo "Compiled source of nearcore"

cd $BRIDGE_SRC
# In local development, this update ethashproof repo
# In npm package, this is safely ignored and ethashproof src is packaged
git submodule update --init --recursive

yarn
echo "Installed CLI dependencies"

cd $BRIDGE_SRC/vendor/ganache
yarn
echo "Installed ganache-cli"

cd $BRIDGE_SRC/vendor/ethashproof
./build.sh
echo 'Compiled ethashproof module'

# Start the pm2 daemon if it is currently not running.
yarn pm2 ping
