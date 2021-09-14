help:
	@echo ======================================Local dev=====================================
	@echo 1 run "make init-yarn" install node packages.
	@echo 2 run "make gen-contarcts" generate ethereum contracts.
	@echo 3 run "make setup-clean-and-prepare" clean and prepare local env.
	@echo 4 run "make start-local-near-and-ganache-nodes" start nearup and ganache.
	@echo 5 run "make deploy-full-contracts" deploy near and eth contracts.
	@echo 6 run "make start-relayer" start relayers.
	@echo 7 run "make stop-all" stop relayers.
	@echo
	@echo ======================================Build Near Contrats=====================================
	@echo "make bsc-build-client" build bsc client near contract.
	@echo "make bsc-build-prover" build bsc prover near contract.
	@echo "make eth-build-client" build eth client near contract.
	@echo "make eth-build-prover" build eth prover near contract.
	@echo
	@echo ======================================Run Near Tests=====================================
	@echo "make bsc-test-client" run tests bsc client
	@echo "make bsc-test-prover" run tests bsc prover
	@echo "make eth-test-client" run tests eth client
	@echo "make eth-test-prover" run tests eth prover
	@echo


# ===============================Init==============================

init-yarn:
	yarn
	yarn install

# ===============================Local==============================

# generate ether contracts
gen-contracts:
	cd contracts/eth/nearbridge/ && yarn && yarn build
	cd contracts/eth/nearprover/ && yarn && yarn build

setup-clean-and-prepare:
	cli/index.js clean
	cli/index.js prepare

# start near blockchain and connect with ganache.
start-local-near-and-ganache-nodes:
	cli/index.js start near-node
	cli/index.js start ganache

# ===============================Deploy contracts localy==============================

# deploy contracts to testnets NEAR and BSC
deploy-full-contracts:
	cli/index.js init-near-contracts
	cli/index.js init-eth-ed25519
	cli/index.js init-eth-client
	cli/index.js init-eth-prover
	cli/index.js init-eth-erc20
	cli/index.js init-eth-locker
	cli/index.js init-near-token-factory

# ===============================Relayers==============================

# start relayers
start-relayer:
	cli/index.js start eth2near-relay
	cli/index.js start near2eth-relay
	cli/index.js start bridge-watchdog
	pm2 logs

# stop relayers
stop-all:
	cli/index.js stop all

# ===============================Build NEAR Contracts==============================

# build bsc near client
bsc-build-client:
	cd contracts/near/eth-client && sudo ./build.sh bsc

# build bsc near prover
bsc-build-prover:
	cd contracts/near/eth-prover && sudo ./build.sh bsc

# build eth near client
eth-build-client:
	cd contracts/near/eth-client && sudo ./build.sh

# build eth near prover
eth-build-prover:
	cd contracts/near/eth-prover && sudo ./build.sh

# ===============================Run tests==============================

# test bsc near client
bsc-test-client:
	cd contracts/near/eth-client && ./test.sh bsc

# test bsc near prover
bsc-test-prover:
	cd contracts/near/eth-prover && ./test.sh bsc

# test eth near client
eth-test-client:
	cd contracts/near/eth-client && ./test.sh

# test eth near prover
eth-test-prover:
	cd contracts/near/eth-prover && ./test.sh

.PHONY: help \
		init-yarn \
		gen-contracts \
		setup-clean-and-prepare \
		start-local-near-and-ganache-nodes \
		deploy-full-contracts \
		deploy-full-contracts \
		start-relayer \
		stop-all \
		bsc-build-client \
		bsc-build-prover \
		eth-build-client \
		eth-build-prover \
		bsc-test-client \
		bsc-test-prover \
		eth-test-client \
		eth-test-prover
