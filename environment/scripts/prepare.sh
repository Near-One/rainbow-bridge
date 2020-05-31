#!/bin/bash
set -euo pipefail

RAINBOWUP_DIR=~/.rainbowup

LOCAL_BRIDGE_SRC=""
LOCAL_CORE_SRC=""

CORE_SRC=~/.rainbowup/core
BRIDGE_SRC=~/.rainbowup/bridge

ARGUMENT_LIST=(
    "source"
    "nearcore_source"
)

if [ -d $RAINBOWUP_DIR ]; then
	echo "~/.rainbowup already exists. Please run the clean"
	exit 1	
fi

mkdir -p $RAINBOWUP_DIR
# read arguments
opts=$(getopt \
    --longoptions "$(printf "%s:," "${ARGUMENT_LIST[@]}")" \
    --name "$(basename "$0")" \
    --options "" \
    -- "$@"
)
eval set --$opts
while [[ $# -gt 0 ]] ; do
  case "$1" in
    --source)
      LOCAL_BRIDGE_SRC=$2
      shift 2
      ;;
    --nearcore_source)
      LOCAL_CORE_SRC=$2
      shift 2
      ;;
    *)
      break
      ;;
  esac
done

if test -z "$LOCAL_CORE_SRC"
then
echo "near-core repo not specified..."
git clone "https://github.com/nearprotocol/nearcore" $CORE_SRC
else
echo "Copying the specified local repo from ${LOCAL_CORE_SRC} to ${CORE_SRC}"
cp -r $LOCAL_CORE_SRC $CORE_SRC 
fi

if test -z "$LOCAL_BRIDGE_SRC"
then
echo "rainbow-bridge repo not specified..."
git clone "https://github.com/near/rainbow-bridge/" $BRIDGE_SRC
else
cp -r $LOCAL_BRIDGE_SRC $BRIDGE_SRC
fi

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

cd $BRIDGE_SRC/environment/vendor/ethashproof
./build.sh
echo 'Compiled ethashproof module'
