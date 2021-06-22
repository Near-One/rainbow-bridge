help:
	@echo Deploy rainbow bridge
	@echo 1 run "make init" first time only and one time.
	@echo 2 run "make start-bsc" or "make start-ethash"
	@echo 3 run "make gen-contarcts"
	@echo 4 run "make deploy-contarcts"
	@echo 5 run "make start-relayer"
	@echo 6 run "stop-all"

init:
	yarn
	yarn install

# start near bc and connect with binance test net.
start-ethash:
	cli/index.js clean
	cli/index.js prepare --core-src ${HOME}/Desktop/core
	cli/index.js start near-node
	cli/index.js start ganache

# start near bc and connect with binance test net.
start-bsc:
	cli/index.js clean
	cli/index.js prepare --core-src ${HOME}/Desktop/core
	cli/index.js start near-node
	cli/index.js start binance-smart-chain

# generate ether contracts
gen-contracts:
	cd contracts/eth/nearbridge/ && yarn && yarn build
	cd contracts/eth/nearprover/ && yarn && yarn build

# deploy contracts
deploy-contracts:
	cli/index.js init-near-contracts
	cli/index.js init-eth-ed25519
	cli/index.js init-eth-client --eth-client-lock-eth-amount 1000 --eth-client-lock-duration 10
	cli/index.js init-eth-prover
	cli/index.js init-eth-erc20
	cli/index.js init-eth-locker
	cli/index.js init-near-token-factory

start-relayer:
	cli/index.js start eth2near-relay
	cli/index.js start near2eth-relay 
	cli/index.js start bridge-watchdog
	pm2 logs

stop-all:
	cli/index.js stop all

build-eth-client:
	cd contracts/near/eth-client && ./build.sh

.PHONEY: help init start-ethash start-bsc gen-contracts deploy-contracts start-relayer stop-all build-eth-client