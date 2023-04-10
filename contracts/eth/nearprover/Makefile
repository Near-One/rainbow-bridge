.PHONY: test

NETWORK=goerli
LEDGER_KEY_PATH=
NEAR_ON_ETH_CLIENT_ADDRESS=
NEW_ADMIN=
NUM_SLOTS=5

test:
	yarn hardhat test

deploy-prover-with-mock-bridge:
	yarn hardhat run --network ${NETWORK} utils/deploy_contracts.js

upgrade-provers-bridge-address-to:
	yarn hardhat --network ${NETWORK} upgrade-provers-bridge-address-to --prover ${PROVER} --new-bridge ${NEW_BRIDGE}

upgrade-provers-bridge-address-to-with-ledger:
	yarn hardhat --network ${NETWORK} upgrade-provers-bridge-address-to --prover ${PROVER} --new-bridge ${NEW_BRIDGE} \
	--ledger-key-path "${LEDGER_KEY_PATH}"

get-provers-bridge-address:
	yarn hardhat --network ${NETWORK} get-provers-bridge-address --prover ${PROVER}

get-slots-data:
	yarn hardhat --network ${NETWORK} get-slots-data --contract-address ${ADDRESS} --num-of-slots-to-display ${NUM_SLOTS}

upgrade-admin:
	yarn hardhat --network ${NETWORK} upgrade-admin-to --contract-address ${ADDRESS} --current-admin-address ${CURRENT_ADMIN} --new-admin-address ${NEW_ADMIN} --slot ${SLOT}

upgrade-admin-with-ledger:
	yarn hardhat --network ${NETWORK} upgrade-admin-to --contract-address ${ADDRESS} --current-admin-address ${CURRENT_ADMIN} --new-admin-address ${NEW_ADMIN} --slot ${SLOT} --ledger-key-path "${LEDGER_KEY_PATH}"
