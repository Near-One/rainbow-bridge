const Web3 = require('web3');
const nearlib = require('near-api-js');
const fs = require('fs');
// @ts-ignore
const bs58 = require('bs58');
// @ts-ignore
const { toBuffer } = require('eth-util-lite');
const { RainbowConfig } = require('./config');
const { BN } = require('ethereumjs-util');
const { sleep, web3GetBlock } = require('../lib/robust');

/// Maximum number of retries a Web3 method call will perform.
const MAX_WEB3_RETRIES = 1000;

function borshify (block) {
    return Buffer.concat([
        bs58.decode(block.prev_block_hash),
        bs58.decode(block.next_block_inner_hash),
        Buffer.concat([
            // @ts-ignore
            Web3.utils.toBN(block.inner_lite.height).toBuffer('le', 8),
            bs58.decode(block.inner_lite.epoch_id),
            bs58.decode(block.inner_lite.next_epoch_id),
            bs58.decode(block.inner_lite.prev_state_root),
            bs58.decode(block.inner_lite.outcome_root),
            // @ts-ignore
            Web3.utils.toBN(block.inner_lite.timestamp).toBuffer('le', 8),
            bs58.decode(block.inner_lite.next_bp_hash),
            bs58.decode(block.inner_lite.block_merkle_root),
        ]),
        bs58.decode(block.inner_rest_hash),

        Buffer.from([1]),
        // @ts-ignore
        Web3.utils.toBN(block.next_bps.length).toBuffer('le', 4),
        Buffer.concat(
            block.next_bps.map(nextBp => Buffer.concat([
                // @ts-ignore
                Web3.utils.toBN(nextBp.account_id.length).toBuffer('le', 4),
                Buffer.from(nextBp.account_id),
                nextBp.public_key.substr(0, 8) === 'ed25519:' ? Buffer.from([0]) : Buffer.from([1]),
                bs58.decode(nextBp.public_key.substr(8)),
                // @ts-ignore
                Web3.utils.toBN(nextBp.stake).toBuffer('le', 16),
            ])),
        ),

        // @ts-ignore
        Web3.utils.toBN(block.approvals_after_next.length).toBuffer('le', 4),
        Buffer.concat(
            block.approvals_after_next.map(
                signature => signature === null
                    ? Buffer.from([0])
                    : Buffer.concat([
                        Buffer.from([1]),
                        signature.substr(0, 8) === 'ed25519:' ? Buffer.from([0]) : Buffer.from([1]),
                        bs58.decode(signature.substr(8)),
                    ]),
            ),
        ),
    ]);
}

class Near2EthRelay {
    async initialize () {
        // @ts-ignore
        this.web3 = new Web3(RainbowConfig.getParam('eth-node-url'));
        this.ethMasterAccount =
            this.web3.eth.accounts.privateKeyToAccount(RainbowConfig.getParam('eth-master-sk'));
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
                while (!lightClientBlock) {
                    // @ts-ignore
                    lightClientBlock = await this.near.connection.provider.sendJsonRpc('next_light_client_block', [lastFinalBlockHash]);
                    if (!lightClientBlock) {
                        await sleep(300);
                    }
                }
                console.log('Initializing with block');
                console.log(`${JSON.stringify(lightClientBlock)}`);
                const borshBlock = borshify(lightClientBlock);
                // @ts-ignore
                const _tx = await this.clientContract.methods.initWithBlock(borshBlock).send({
                    from: this.ethMasterAccount,
                    gas: 1000000,
                    handleRevert: true,
                });
            }
            console.log('Client is initialized.');
        } catch (txRevertMessage) {
            console.log('Failure.');
            console.log(txRevertMessage.toString());
            process.exit(1);
        }
    }

    async run () {
        process.send('ready');
        const clientContract = this.clientContract;
        const web3 = this.web3;
        const near = this.near;
        const ethMasterAccount = this.ethMasterAccount;
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
                const latestBlock = await web3GetBlock(web3, 'latest');
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
            const lightClientBlock = await near.connection.provider.sendJsonRpc('next_light_client_block', [clientBlockHash]);
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
                        gas: 1000000,
                        handleRevert: true,
                    });
                    break;
                } catch (err) {
                    console.log(`Encountered Web3 error while submitting light client block ${err}`);
                    await sleep(1000);
                }
            }

            await step();
        };

        await step();
    }
}

exports.Near2EthRelay = Near2EthRelay;
