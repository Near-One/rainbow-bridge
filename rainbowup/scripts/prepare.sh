#!/bin/bash
set -euo pipefail

BRIDGE_SRC=""
ORE_SRC=""
ARGUMENT_LIST=(
    "source"
    "nearcore_source"
)
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
      BRIDGE_SRC=$2
      shift 2
      ;;
    --nearcore_source)
      CORE_SRC=$2
      shift 2
      ;;
    *)
      break
      ;;
  esac
done



if test -z "$CORE_SRC"
then
echo "near-core repo not specified..."
CORE_SRC=~/.rainbowup/core
git clone "https://github.com/nearprotocol/nearcore" $CORE_SRC
fi

if test -z "$BRIDGE_SRC"
then
echo "rainbow-bridge repo not specified..."
BRIDGE_SRC=~/.rainbowup/bridge
git clone "https://github.com/near/rainbow-bridge/" $BRIDGE_SRC
cd $BRIDGE_SRC
git submodule update --init --recursive
fi

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
