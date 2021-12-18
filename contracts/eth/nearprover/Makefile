.PHONY: test

NETWORK=goerli
LEDGER_KEY_PATH=''

test:
	yarn hardhat test

deploy-prover-with-mock-bridge:
	yarn hardhat run --network ${NETWORK} utils/deploy_contracts.js

upgrade-provers-bridge-address-to:
	yarn hardhat --network ${NETWORK} upgrade-provers-bridge-address-to --prover ${PROVER} --new-bridge ${NEW_BRIDGE}

upgrade-provers-bridge-address-to-with-ledger:
	yarn hardhat --network ${NETWORK} upgrade-provers-bridge-address-to --prover ${PROVER} --new-bridge ${NEW_BRIDGE} \
	--ledger-key-path ${LEDGER_KEY_PATH}

get-provers-bridge-address:
	yarn hardhat --network ${NETWORK} get-provers-bridge-address --prover ${PROVER}

