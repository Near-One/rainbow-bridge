#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -o errexit

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
NODE_URL="https://rpc.nearprotocol.com"

# The master account ID to create accounts
NEAR_MASTER_ACCOUNT_ID="ethrelay"
# The account ID to push blocks
NEAR_RELAYER_ACCOUNT_ID="ethrelay-pusher"
# The account ID used as ETH bridge
NEAR_ETHBRIDGE_ACCOUNT_ID="ethbridge"

echo "Creating account to push blocks:"
NODE_ENV=local yarn run near --nodeUrl=$NODE_URL --homeDir "$DIR/.near" --keyPath "$DIR/.near/validator_key.json" create_account $NEAR_RELAYER_ACCOUNT_ID --masterAccount=NEAR_MASTER_ACCOUNT_ID --initialBalance 100000000 || echo "Skip creating ethbridge accout"
echo "Creating account for smart contract:"
NODE_ENV=local yarn run near --nodeUrl=$NODE_URL --homeDir "$DIR/.near" --keyPath "$DIR/.near/validator_key.json" create_account $NEAR_ETHBRIDGE_ACCOUNT_ID --masterAccount=NEAR_MASTER_ACCOUNT_ID --initialBalance 100000000 || echo "Skip creating ethbridge accout"
echo "Deploying smart contract:"
NODE_ENV=local yarn run near --nodeUrl=$NODE_URL --homeDir "$DIR/.near" --keyPath "$DIR/.near/validator_key.json" deploy --contractName $NEAR_ETHBRIDGE_ACCOUNT_ID --wasmFile "$DIR/../../ethbridge/res/eth_bridge.wasm" || echo "Skip deploying ethbridge smart contract"

# Launch EthRelay
while :
do
  BRIDGE_VALIDATE_ETHASH=true \
    NEAR_NODE_URL=$NODE_URL \
    NEAR_NODE_NETWORK_ID=default \
    NEAR_RELAYER_ACCOUNT_ID=$NEAR_RELAYER_ACCOUNT_ID \
    NEAR_ETHBRIDGE_ACCOUNT_ID=$NEAR_ETHBRIDGE_ACCOUNT_ID \
    ETHEREUM_NODE_URL="wss://mainnet.infura.io/ws/v3/b5f870422ee5454fb11937e947154cd2" \
    node "$DIR/../index.js" || echo "!!! Seems the relayer has failed. Going to try again in a minute."
  sleep 60
done
