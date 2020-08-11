const Web3 = require('web3');
const nearlib = require('near-api-js');
const fs = require('fs');
// @ts-ignore
const bs58 = require('bs58');
// @ts-ignore
const { toBuffer } = require('eth-util-lite');
const { RainbowConfig } = require('./config');
const { BN } = require('ethereumjs-util');
const { sleep, RobustWeb3, normalizeEthKey } = require('../lib/robust');
const { borshify, borshifyInitialValidators } = require('../lib/borsh')

/// Maximum number of retries a Web3 method call will perform.
const MAX_WEB3_RETRIES = 1000;

class Near2EthRelay {
    async initialize() {
        // @ts-ignore
        this.robustWeb3 = new RobustWeb3(RainbowConfig.getParam('eth-node-url'));
        this.web3 = this.robustWeb3.web3;
        this.ethMasterAccount =
            this.web3.eth.accounts.privateKeyToAccount(normalizeEthKey(RainbowConfig.getParam('eth-master-sk')));
        this.web3.eth.accounts.wallet.add(this.ethMasterAccount);
        this.web3.eth.defaultAccount = this.ethMasterAccount.address;
        this.ethMasterAccount = this.ethMasterAccount.address;

        const keyStore = new nearlib.keyStores.InMemoryKeyStore();
        this.near = await nearlib.connect({
            nodeUrl: RainbowConfig.getParam('near-node-url'),
            networkId: RainbowConfig.getParam('near-network-id'),
            deps: {
                keyStore: keyStore,
            },
        });

        // Declare Near2EthClient contract.
        this.clientContract = new this.web3.eth.Contract(
            // @ts-ignore
            JSON.parse(fs.readFileSync(RainbowConfig.getParam('near2eth-client-abi-path'))),
            RainbowConfig.getParam('near2eth-client-address'), {
            from: this.ethMasterAccount,
            handleRevert: true,
        },
        );

        // Check if initialization is needed.
        try {
            console.log('Checking whether client is initialized.');
            const isInitialized = await this.clientContract.methods.initialized().call();
            if (!isInitialized) {
                console.log('Client is not initialized. Initializing.');
                // Get most recent block from Near blockchain.
                const status = await this.near.connection.provider.status();
                // Get the block two blocks before that, to make sure it is final.
                const headBlock = await this.near.connection.provider.block({ blockId: status.sync_info.latest_block_height });
                // @ts-ignore
                const lastFinalBlockHash = headBlock.header.last_final_block;
                // The finalized block is not immediately available so we wait for it to become available.
                let lightClientBlock = null;
                let currentValidators = null;
                while (!lightClientBlock) {
                    // @ts-ignore
                    currentValidators = await this.near.connection.provider.validators(null);
                    if (!currentValidators) {
                        await sleep(300);
                        continue;
                    }
                    lightClientBlock = await this.near.connection.provider.sendJsonRpc('next_light_client_block', [lastFinalBlockHash]);
                    if (!lightClientBlock) {
                        await sleep(300);
                        continue;
                    }
                    // Because fetch currentValidators and lightClientBlock isn't atomic, it's possible we happen to
                    // fetch lightClentBlock cross epoch boundary. Fetch another time to ensure that's not the case.
                    // @ts-ignore
                    let currentValidatorsNow = await this.near.connection.provider.validators(null);
                    if (!currentValidatorsNow || currentValidatorsNow.epoch_start_height != currentValidators.epoch_start_height) {
                        await sleep(300);
                        continue;
                    }
                }
                console.log('Initializing with validators');
                console.log(`${JSON.stringify(currentValidators.current_validators)}`);
                const borshInitialValidators = borshifyInitialValidators(currentValidators.current_validators);
                // @ts-ignore
                let gasPrice = new BN(await this.web3.eth.getGasPrice()).mul(new BN(RainbowConfig.getParam('eth-gas-multiplier')));
                let err;
                for (let i = 0; i < 10; i++) {
                    try {
                        await this.clientContract.methods.initWithValidators(borshInitialValidators).send({
                            from: this.ethMasterAccount,
                            gas: 4000000,
                            handleRevert: true,
                            gasPrice,
                        });
                    } catch (e) {
                        if (e.message.includes('replacement transaction underpriced')) {
                            gasPrice = gasPrice.mul(new BN(11)).div(new BN(10));
                            continue;
                        }
                        err = e;
                    }
                    break;
                }
                if (err) {
                    console.log('Failure');
                    console.log(err);
                    process.exit(1);
                }

                console.log('Initializing with block');
                console.log(`${JSON.stringify(lightClientBlock)}`);
                const borshBlock = borshify(lightClientBlock);
                for (let i = 0; i < 10; i++) {
                    try {
                        await this.clientContract.methods.initWithBlock(borshBlock).send({
                            from: this.ethMasterAccount,
                            gas: 4000000,
                            handleRevert: true,
                            gasPrice: new BN(await this.web3.eth.getGasPrice()).mul(new BN(RainbowConfig.getParam('eth-gas-multiplier'))),
                        });
                    } catch (e) {
                        if (e.message.includes('replacement transaction underpriced')) {
                            gasPrice = gasPrice.mul(new BN(11)).div(new BN(10));
                            continue;
                        }
                        err = e;
                    }
                    break;
                }
                if (err) {
                    console.log('Failure');
                    console.log(err);
                    process.exit(1);
                }
            }
            console.log('Client is initialized.');
        } catch (txRevertMessage) {
            console.log('Failure.');
            console.log(txRevertMessage.toString());
            process.exit(1);
        }
    }

    async run() {
        // process.send('ready');
        const clientContract = this.clientContract;
        const robustWeb3 = this.robustWeb3;
        const near = this.near;
        const ethMasterAccount = this.ethMasterAccount;
        const web3 = this.web3;
        const step = async function () {
            // Sleep until the last Near block becomes valid.
            let lastClientBlock;
            let clientBlockHash;
            while (true) {
                lastClientBlock = await clientContract.methods.last().call();
                const clientBlockHeight = lastClientBlock.height;
                const clientBlockHashHex = await clientContract.methods.blockHashes(clientBlockHeight).call();
                clientBlockHash = bs58.encode(toBuffer(clientBlockHashHex));
                console.log(`Current light client head is: hash=${clientBlockHash}, height=${clientBlockHeight}`);
                const latestBlock = await robustWeb3.getBlock('latest');
                if (latestBlock.timestamp >= lastClientBlock.validAfter) {
                    console.log('Block is valid.');
                    break;
                } else {
                    const sleepSec = (lastClientBlock.validAfter - latestBlock.timestamp);
                    console.log(`Block is not valid yet. Sleeping ${sleepSec} seconds.`);
                    await sleep(sleepSec * 1000);
                }
            }

            // Check whether master account has enough balance at stake.
            const lockEthAmount = await clientContract.methods.lock_eth_amount().call();
            const balance = await clientContract.methods.balanceOf(ethMasterAccount).call();
            if (balance === '0') {
                console.log(`The sender account does not have enough stake. Transferring ${lockEthAmount} wei.`);
                // @ts-ignore
                let _depositTx;
                for (let i = 0; i <= MAX_WEB3_RETRIES; i++) {
                    if (i === MAX_WEB3_RETRIES) {
                        console.error(`Failed ${MAX_WEB3_RETRIES} times`);
                        process.exit(1);
                    }
                    try {
                        _depositTx = await clientContract.methods.deposit().send({
                            from: ethMasterAccount,
                            gas: 1000000,
                            handleRevert: true,
                            value: (new BN(lockEthAmount)),
                            gasPrice: new BN(await web3.eth.getGasPrice()).mul(new BN(RainbowConfig.getParam('eth-gas-multiplier'))),
                        });
                        break;
                    } catch (err) {
                        console.log(`Encountered Web3 error while depositing stake ${err}`);
                        await sleep(1000);
                    }
                }
                console.log('Transferred.');
            }

            // Get new light client block.
            // @ts-ignore
            let lightClientBlock;
            for (let i = 0; i <= MAX_WEB3_RETRIES; i++) {
                if (i === MAX_WEB3_RETRIES) {
                    console.error(`Failed ${MAX_WEB3_RETRIES} times`);
                    process.exit(1);
                }
                try {
                    lightClientBlock = await near.connection.provider.sendJsonRpc('next_light_client_block', [clientBlockHash]);
                    break;
                } catch (err) {
                    console.log(`Encountered error while requesting light client block ${err}`);
                    await sleep(1000);
                }
            }
            console.log('Adding block');
            console.log(`${JSON.stringify(lightClientBlock)}`);

            const borshBlock = borshify(lightClientBlock);
            for (let i = 0; i <= MAX_WEB3_RETRIES; i++) {
                if (i === MAX_WEB3_RETRIES) {
                    console.error(`Failed ${MAX_WEB3_RETRIES} times`);
                    process.exit(1);
                }
                try {
                    await clientContract.methods.addLightClientBlock(borshBlock).send({
                        from: ethMasterAccount,
                        gas: 4000000,
                        handleRevert: true,
                        gasPrice: new BN(await web3.eth.getGasPrice()).mul(new BN(RainbowConfig.getParam('eth-gas-multiplier'))),
                    });
                    break;
                } catch (err) {
                    console.log(`Encountered Web3 error while submitting light client block ${err}`);
                    await sleep(1000);
                }
            }

            let sleepTime = (new BN(RainbowConfig.getParam('near2eth-relay-delay'))).toNumber();
            if (sleepTime > 0) {
                console.log(`Sleeping for ${sleepTime} seconds.`);
                await sleep(sleepTime * 1000);
            }
            await step();
        };

        await step();
    }
}

exports.Near2EthRelay = Near2EthRelay;
exports.borshify = borshify;
