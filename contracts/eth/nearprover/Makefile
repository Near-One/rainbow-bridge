.PHONY: test

NETWORK=goerli
LEDGER_KEY_PATH=''
NEAR_ON_ETH_CLIENT_ADDRESS=
NEW_ADMIN=

test:
	yarn hardhat test

deploy-prover-with-mock-bridge:
	yarn hardhat run --network ${NETWORK} utils/deploy_contracts.js

upgrade-provers-bridge-address-to:
	yarn hardhat --network ${NETWORK} upgrade-provers-bridge-address-to --prover ${PROVER} --new-bridge ${NEW_BRIDGE}

upgrade-provers-bridge-address-to-with-ledger:
	yarn hardhat --network ${NETWORK} upgrade-provers-bridge-address-to --prover ${PROVER} --new-bridge ${NEW_BRIDGE} \
	--ledger-key-path ${LEDGER_KEY_PATH}

upgrade-near-on-eth-client-admin-to:
	yarn hardhat --network ${NETWORK} upgrade-near-on-eth-client-admin-to --near-on-eth-client-address ${NEAR_ON_ETH_CLIENT_ADDRESS} --new-admin ${NEW_ADMIN}

upgrade-near-on-eth-client-admin-to-with-ledger:
	yarn hardhat --network ${NETWORK} upgrade-near-on-eth-client-admin-to --near-on-eth-client-address ${NEAR_ON_ETH_CLIENT_ADDRESS} --new-admin ${NEW_ADMIN} --ledger-key-path ${LEDGER_KEY_PATH}

get-provers-bridge-address:
	yarn hardhat --network ${NETWORK} get-provers-bridge-address --prover ${PROVER}

