#!/bin/bash

CORE_SRC="~/.rainbowup/core"
BRIDGE_SRC="~/.rainbowup/bridge"

while : ; do
  case "$1" in 
    --nearcore_source)
       [ -n "${CORE_SRC}" ] && usage
       $CORE_SRC="$2"
       shift 2 ;;
    --source)
       [ -n "${BRIDGE_SRC}" ] && usage
       BRIDGE_SRC="$2"
       shift 2 ;;
    *)
       break ;;
  esac
done

cd $CORE_SRC && cargo build --package neard --bin neard
echo "Compiled source of nearcore"

cd $BRIDGE_SRC/libs-rs && ./build_all.sh
echo "Compiled Rust contracts"

cd $BRIDGE_SRC/libs-sol && ./build_all.sh
echo "Built Solidity contracts"

cd $BRIDGE_SRC/environment && yarn

cd $BRIDGE_SRC/environment/vendor/ethashproof && ./build.sh
echo 'Compiled ethashproof module'

