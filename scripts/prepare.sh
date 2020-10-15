#!/bin/bash
set -euo pipefail

eval RAINBOW_DIR=~/.rainbow

eval CORE_SRC=~/.rainbow/core
SCRIPTS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" 2>&1 && pwd )"
eval BRIDGE_SRC=${SCRIPTS_DIR}/..
eval LIBS_SOL_SRC=${BRIDGE_SRC}/node_modules/rainbow-bridge-sol
eval LIBS_RS_SRC=${BRIDGE_SRC}/node_modules/rainbow-bridge-rs

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

unameOut="$(uname -s)"
case "${unameOut}" in
    Linux*)     machine=Linux;;
    Darwin*)    machine=Mac;;
    *)          machine="UNKNOWN:${unameOut}"
esac

if [[ $machine == 'Linux' ]]
then
	mkdir -p $CORE_SRC/target/debug 
  wget "https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore/Linux/master/near" -O $CORE_SRC/target/debug/neard -q
  chmod +x $CORE_SRC/target/debug/neard
else

if test -z "$LOCAL_CORE_SRC"
then
  echo "near-core home not specified..."
  git clone "https://github.com/nearprotocol/nearcore" $CORE_SRC
  eval CURR_DIR=$(pwd)
  cd $CURR_DIR
else
  echo "Linking the specified local repo from ${LOCAL_CORE_SRC} to ${CORE_SRC}"
  if [ -L $CORE_SRC ]
  then
    unlink $CORE_SRC
  fi
  ln -sf $LOCAL_CORE_SRC $CORE_SRC
fi

cd $CORE_SRC
cargo build --package neard --bin neard
echo "Compiled source of nearcore"

fi


cd $BRIDGE_SRC
# In local development, this update ethashproof repo
# In npm package, ethashproof src is packaged so this is skipped.
if [ -d .git ]; then
    git submodule update --init --recursive
fi

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
