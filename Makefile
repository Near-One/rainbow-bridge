help:
	@echo Deploy rainbow bridge
	@echo 1 run "make init" first time only and one time.
	@echo 2 run "make startup"
	@echo 3 run "make gen-contarcts"
	@echo 4 run "make deploy-contarcts"
	@echo 5 run "make start-relayer"
	@echo 6 run "stop-all"

init:
	yarn
	yarn install

startup:
	cli/index.js clean
	cli/index.js prepare
	cli/index.js start near-node
	cli/index.js start ganache


gen-contracts:
	cd contracts/eth/nearbridge/ && yarn && yarn build
	cd contracts/eth/nearprover/ && yarn && yarn build

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
	cli/index.js start near2eth-relay --eth-master-sk 0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501201
	cli/index.js start bridge-watchdog --eth-master-sk 0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501202
	pm2 logs

stop-all:
	cli/index.js stop all


.PHONEY: help init startup gen-contracts deploy-contracts start-relayer stop-all 