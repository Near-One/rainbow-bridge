#!/usr/bin/env bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

source $DIR/../../scripts/start_ganache.sh
source $DIR/../../scripts/start_nearcore.sh

# Exit script as soon as a command fails.
set -o errexit

start_ganache_if_needed
#start_nearcore_if_needed
$DIR/../../ethrelay/scripts/run_ganache_to_localnet.sh &
ethrelay_pid=$!
sleep 10

# Executes cleanup function at script exit.
trap_add cleanup_ethrelay EXIT

cleanup_ethrelay() {
    # Kill the ganache instance that we started (if we started one and if it's still running).
    if [ -n "$ethrelay_pid" ] && ps -p $ethrelay_pid > /dev/null; then
        kill $ethrelay_pid
    fi
}

oz compile
ETH_CONTRACT_ADDRESS=$(oz deploy Emitter --kind regular --network development --silent --no-interactive)
EXH_TX=$(oz send-tx --to $ETH_CONTRACT_ADDRESS --method "emitEvent(uint256,uint256,uint256)" --args 1,2,3 --network development)

$DIR/../build.sh

ETH_NODE_URL="http://localhost:9545"
NODE_URL="http://localhost:3030"
NODE_ROOT="$DIR/../../scripts/.near"

# The master account ID to create accounts
NEAR_MASTER_ACCOUNT_ID="ethrelay"
# The account ID to push blocks
NEAR_CALLER_ACCOUNT_ID="ethprover-caller"
# The account ID used as ETH Prover
NEAR_ETHPROVER_ACCOUNT_ID="ethprover"
# The account ID used as ETH bridge
NEAR_ETHBRIDGE_ACCOUNT_ID="ethbridge"

echo "Creating account to call prover:"
NODE_ENV=local yarn run near --nodeUrl=$NODE_URL --homeDir $NODE_ROOT --keyPath "$NODE_ROOT/validator_key.json" create_account $NEAR_CALLER_ACCOUNT_ID --masterAccount=$NEAR_MASTER_ACCOUNT_ID --initialBalance 100000000 || echo "Skip creating caller accout"
echo "Creating account for smart contract:"
NODE_ENV=local yarn run near --nodeUrl=$NODE_URL --homeDir $NODE_ROOT --keyPath "$NODE_ROOT/validator_key.json" create_account $NEAR_ETHPROVER_ACCOUNT_ID --masterAccount=$NEAR_MASTER_ACCOUNT_ID --initialBalance 100000000 || echo "Skip creating ethprover accout"
echo "Deploying smart contract:"
NODE_ENV=local yarn run near --nodeUrl=$NODE_URL --homeDir $NODE_ROOT --keyPath "$NODE_ROOT/validator_key.json" deploy --contractName $NEAR_ETHPROVER_ACCOUNT_ID --wasmFile "$DIR/../res/eth_prover.wasm" || echo "Skip deploying ethprover smart contract"

NEAR_BLOCK_NUMBER=9591235
NEAR_EXPECTED_BLOCK_HASH="0x4825597982da98143abc4720083439ebd2e698637a2edd7565a2e198b18f17cd"

# Launch EthProver call
NEAR_NODE_URL=$NODE_URL \
  NEAR_BLOCK_NUMBER=$NEAR_BLOCK_NUMBER \
  NEAR_EXPECTED_BLOCK_HASH=$NEAR_EXPECTED_BLOCK_HASH \
  NEAR_NODE_NETWORK_ID=local \
  NEAR_CALLER_ACCOUNT_ID=$NEAR_CALLER_ACCOUNT_ID \
  NEAR_ETHPROVER_ACCOUNT_ID=$NEAR_ETHPROVER_ACCOUNT_ID \
  NEAR_ETHBRIDGE_ACCOUNT_ID=$NEAR_ETHBRIDGE_ACCOUNT_ID \
  ETH_NODE_URL=$ETH_NODE_URL \
  ETH_CONTRACT_ADDRESS=$ETH_CONTRACT_ADDRESS \
  node "$DIR/../index.js"

# NEAR_BLOCK_NUMBER=9591234
# NEAR_EXPECTED_BLOCK_HASH="0x4825597982da98143abc4720083439ebd2e698637a2edd7565a2e198b18f17cd"

# echo "Bad block"
# # Launch EthProver call
# NEAR_NODE_URL=$NODE_URL \
#   NEAR_BLOCK_NUMBER=$NEAR_BLOCK_NUMBER \
#   NEAR_EXPECTED_BLOCK_HASH=$NEAR_EXPECTED_BLOCK_HASH \
#   NEAR_NODE_NETWORK_ID=default \
#   NEAR_CALLER_ACCOUNT_ID=$NEAR_CALLER_ACCOUNT_ID \
#   NEAR_ETHPROVER_ACCOUNT_ID=$NEAR_ETHPROVER_ACCOUNT_ID \
#   NEAR_ETHBRIDGE_ACCOUNT_ID=$NEAR_ETHBRIDGE_ACCOUNT_ID \
#   node "$DIR/index.js"
