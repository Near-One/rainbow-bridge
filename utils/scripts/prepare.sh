#!/bin/bash
set -xeo pipefail

eval RAINBOW_DIR=~/.rainbow

eval CORE_SRC=~/.rainbow/core
SCRIPTS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" 2>&1 && pwd )"
eval BRIDGE_SRC=${SCRIPTS_DIR}/../..
eval LIBS_SOL_SRC=${BRIDGE_SRC}/node_modules/rainbow-bridge-sol
eval LIBS_RS_SRC=${BRIDGE_SRC}/node_modules/rainbow-bridge-rs
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

pip3 install nearup --upgrade --user

unameOut="$(uname -s)"
case "${unameOut}" in
    Linux*)     MACHINE=Linux;;
    Darwin*)    MACHINE=Mac;;
    CYGWIN*)    MACHINE=Cygwin;;
    MINGW*)     MACHINE=MinGw;;
    *)          MACHINE="UNKNOWN:${unameOut}"
esac

if test -z "$LOCAL_CORE_SRC"
then
  echo "near-core home not specified..."
  if [ "$MACHINE" == "Linux" ]; then
    NEAR_CORE_LATEST_COMMIT="$(git ls-remote git://github.com/near/nearcore.git | grep refs/heads/master | cut -f 1)"
    NEAR_CORE_BINARY_URL="https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore/Linux/master/$NEAR_CORE_LATEST_COMMIT/neard"
    NEAR_CORE_BINARY_DIR="$CORE_SRC/target/debug"
    NEAR_CORE_BINARY_PATH="$NEAR_CORE_BINARY_DIR/neard"
    mkdir -p $NEAR_CORE_BINARY_DIR
    curl $NEAR_CORE_BINARY_URL > $NEAR_CORE_BINARY_PATH
    chmod +x $NEAR_CORE_BINARY_PATH
  else
    git clone "https://github.com/nearprotocol/nearcore" $CORE_SRC
    cd $CORE_SRC
    cargo build --package neard --bin neard
  fi
else
  echo "Linking the specified local repo from ${LOCAL_CORE_SRC} to ${CORE_SRC}"
  ln -s $LOCAL_CORE_SRC $CORE_SRC
  cd $CORE_SRC
  cargo build --package neard --bin neard
fi

mkdir -p $NEARUP_LOGS

cd $BRIDGE_SRC
# In local development, this update ethashproof repo
# In npm package, ethashproof src is packaged so this is skipped.
if [ -d .git ]; then
    git submodule update --init --recursive
fi

yarn
echo "Installed CLI dependencies"

cd $BRIDGE_SRC/testing/vendor/ganache
yarn
echo "Installed ganache-cli"

cd $BRIDGE_SRC/eth2near/ethashproof
./build.sh
echo 'Compiled ethashproof module'

# Start the pm2 daemon if it is currently not running.
yarn pm2 ping
