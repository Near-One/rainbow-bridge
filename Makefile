help:
	@echo Deploy rainbow bridge
	@echo ======================================Local dev=====================================
	@echo 1 run "make init" first time only and one time.
	@echo 2 run "make local-start" to start local development tools:ganache, nearup
	@echo 2 run "make local-start-bsc" to start local development tools:bsc testnet, nearup
	@echo 3 run "make gen-contarcts"
	@echo 4 run "make local-full-contracts"
	@echo 5 run "make start-relayer"
	@echo 6 run "stop-all"
	@echo
	@echo ======================================Testnet======================================
	@echo 1 run "make init-config" copy config testnet file to ${HOME}/.rainbow/config.json
	@echo 2 run "make testnet-full-contracts" Deploy contracts to BSC and NEAR.
	@echo 3 run "make start-relayer" Deploy contracts to BSC and NEAR.
	@echo 4 run "stop-all"
	@echo
	@echo ======================================Test the tesnet bridge======================================
	@echo 1 run "make near-balance" get balance of a near account.
	@echo 2 run "make transfer-eth-to-near" transfer tokens from eth to near.
	@echo 3 run "make transfer-near-to-eth" transfer tokens from near to eth.
	@echo
	

# ===============================Init==============================
init: yarn-init gen-contracts
	
yarn-init:
	yarn
	yarn install

# ===============================Local==============================

# generate ether contracts
gen-contracts:
	cd contracts/eth/nearbridge/ && yarn && yarn build
	cd contracts/eth/nearprover/ && yarn && yarn build

# start near blockchain and connect with ganache.
local-start:
	cli/index.js clean
	cli/index.js prepare
	cli/index.js start near-node 
	cli/index.js start ganache

# start near blockchain and connect with binance test net.
local-start-bsc:
	cli/index.js clean
	cli/index.js prepare
	cli/index.js start near-node
	cli/index.js start binance-smart-chain \
	--eth-node-url https://data-seed-prebsc-1-s1.binance.org:8545 \
	--eth-master-sk 0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200

# deploy full contracts.
local-full-contracts:
	cli/index.js init-near-contracts --num-confirmations 3 --near-prover-contract-path ${PWD}/contracts/near/res/bsc_prover.wasm
	cli/index.js init-eth-ed25519
	cli/index.js init-eth-client --eth-client-lock-duration 2 --eth-client-replace-duration 4
	cli/index.js init-eth-prover
	cli/index.js init-eth-erc20
	cli/index.js init-eth-locker
	cli/index.js init-near-token-factory

# ===============================Testnet==============================

# copy the testnet config file to the ${HOME}/.rainbow/config.json 
init-config:
	mkdir -p ${HOME}/.rainbow
	cp config.json ${HOME}/.rainbow/config.json
	
# deploy contracts to testnets NEAR and BSC
testnet-full-contracts:
	cli/index.js init-near-contracts \
		--near-master-account 0master.testnet \
		--near-master-sk ed25519:Stg3LiwvhuTU5JvUMejBHvmoMxz3473wZHmoiE3j3VVTb1VAeTZRa258wGiiGBMJ5ppdTSP4UDjrjw5PizNch2t \
		--near-client-account n0cli.testnet \
		--near-client-sk ed25519:2eZcTKk6ic9Wk2iXgDu8ok38HNdeua5KeU9LzdpvDGUd5NGFFNNoJUZn2fWxbnWKYYDvhmhV4pqvSV1QVYTXxrSE \
		--near-prover-account n0prv.testnet \
		--near-prover-sk ed25519:2LWJDmCKL4Vy49jKnV4P4Znasvv6ngNNNqwhMuQEfKCUSaus7TQQTK7yDF6QPGgtfkAWJQAiYeGHJKyXhDipd7iJ \
		--near-client-contract-path ${PWD}/contracts/near/res/bsc_eth_client.wasm \
		--near-prover-contract-path ${PWD}/contracts/near/res/bsc_prover.wasm \
		--num-confirmations 3

	cli/index.js init-eth-ed25519
	cli/index.js init-eth-client --eth-client-lock-eth-amount 1000 --eth-client-lock-duration 10
	cli/index.js init-eth-prover \

testnet-factory:
	cli/index.js init-eth-locker \
		--eth-gas-multiplier 2 \
		--near-token-factory-account n0fac.testnet

	cli/index.js init-near-token-factory  \
	--near-prover-account n0prv.testnet \
	--near-token-factory-account n0fac.testnet \
	--near-token-factory-sk ed25519:5sdVyUoFUDg127HvWcjQX3tz5NpysWU6HTeMEL3j3bFSuKWNjwW4NffaqAaJuH9Zi9P4SJcb1wbJjfDFaNzLztnr \
	--eth-erc20-address 0x4bbad5810a29cd5144ca3b6ca15db3664770f604

# ===============================Relayers==============================

# start relayers
start-relayer:
# cli/index.js start eth2near-relay --gas-per-transaction 75000000000000 --total-submit-block 4
	cli/index.js start eth2near-relay --gas-per-transaction 100000000000000 --total-submit-block 3
	cli/index.js start near2eth-relay --eth-master-sk 0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200
	cli/index.js start bridge-watchdog --eth-master-sk 0x6c4f2ebaf0ffa68c9822d7645fd52f365fad8164296323aa9ee4e987120a105c
	pm2 logs

# start relayers
stop-all:
	cli/index.js stop all


# ===============================Build NEAR Contracts==============================

build-bsc-client:
	cd contracts/near/eth-client && sudo ./build.sh bsc

build-bsc-client:
	cd contracts/near/eth-client && sudo ./build.sh bsc

build-bsc-prover:
	cd contracts/near/eth-prover && sudo ./build.sh bsc

# ===============================Run tests==============================

test-bsc-client:
	cd contracts/near/eth-client && ./test.sh bsc

test-bsc-prover:
	cd contracts/near/eth-prover && ./test.sh bsc


# ===============================Test the bsc bridge==============================
near-balance:
	cli/index.js TESTING get-bridge-on-near-balance --near-receiver-account simple10.testnet

transfer-eth-to-near:
	cli/index.js TESTING transfer-eth-erc20-to-near \
		--amount 10 --eth-sender-sk 0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200 \
		--near-receiver-account simple10.testnet \
		--near-master-account simple10.testnet \
		--near-master-sk ed25519:4aFi4332BrFHR2pYXcqsz51P5qL4piHYWYtXggPWxPRxtLxT5veeyfFGyevJpCP7ZW13RzmPa1V2RvkApqYjMXoV

transfer-near-to-eth:
	cli/index.js TESTING transfer-eth-erc20-from-near \
		--amount 1 \
		--near-sender-sk ed25519:4aFi4332BrFHR2pYXcqsz51P5qL4piHYWYtXggPWxPRxtLxT5veeyfFGyevJpCP7ZW13RzmPa1V2RvkApqYjMXoV \
		--near-sender-account simple10.testnet \
		--eth-receiver-address 0xDf08F82De32B8d460adbE8D72043E3a7e25A3B39

.PHONY: help init yarn-init gen-contracts local-start local-start-bsc local-full-contracts init-config testnet-full-contracts start-relayer stop-all build-eth-client build-bsc-client build-eth-prover test-eth-client near-balance transfer-eth-to-near transfer-near-to-eth
