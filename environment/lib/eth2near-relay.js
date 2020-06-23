const Web3 = require('web3');
const exec = require('child_process').exec;
const utils = require('ethereumjs-util');
const BN = require('bn.js');
const blockFromRpc = require('ethereumjs-block/from-rpc');

function execute (command, callback) {
    return new Promise(resolve => exec(command, (error, stdout, stderr) => {
        if (error) {
            console.log(error);
        }
        resolve(stdout);
    }));
}

function web3BlockToRlp (blockData) {
    blockData.difficulty = parseInt(blockData.difficulty, 10);
    blockData.totalDifficulty = parseInt(blockData.totalDifficulty, 10);
    blockData.uncleHash = blockData.sha3Uncles;
    blockData.coinbase = blockData.miner;
    blockData.transactionTrie = blockData.transactionsRoot;
    blockData.receiptTrie = blockData.receiptsRoot;
    blockData.bloom = blockData.logsBloom;
    const blockHeader = blockFromRpc(blockData);
    return utils.rlp.encode(blockHeader.header.raw);
}

class Eth2NearRelay {
    initialize (ethClientContract, ethNodeURL) {
        this.ethClientContract = ethClientContract;
        this.web3 = new Web3(ethNodeURL);
    }

    async run () {
        let last_block_number = (await this.ethClientContract.last_block_number()).toNumber();
        console.log('Contract block number is ' + last_block_number);
        if (last_block_number === 0) {
            // Let's start bridge from current block since it is not initialized
            last_block_number = await this.web3.eth.getBlockNumber();
            console.log('Web3 block number is ' + last_block_number);
        }

        this.subscribeOnBlocksRangesFrom(last_block_number, async (start, stop) => {
            const timeBeforeProofsComputed = Date.now();
            console.log(`Processing ${stop - start + 1} blocks from #${start} to #${stop}`);
            for (let i = start; i <= stop; ++i) {
                let ok = false;
                for (let retryIter = 0; retryIter < 10; retryIter++) {
                    try {
                        const blockRlp = this.web3.utils.bytesToHex(web3BlockToRlp(await this.web3.eth.getBlock(i)));
                        const unparsedBlock = await execute(`./vendor/ethashproof/cmd/relayer/relayer ${blockRlp} | sed -e '1,/Json output/d'`);
                        const block = JSON.parse(unparsedBlock);
                        this.submitBlock(block, i).catch((e) => {
                            console.error(e);
                            process.exit(2);
                        });
                        ok = true;
                        break;
                    } catch (e) {
                        console.log(`Sleeping 0.5sec. Failed at iteration #${retryIter}:`, e);
                        await new Promise((resolve, reject) => {
                            setTimeout(resolve, 500);
                        });
                    }
                }
                if (!ok) {
                    console.error(`Failed to create a proof for a block #${i}`);
                    process.exit(3);
                }
            }
            console.log(
                'Proofs computation took ' + Math.trunc((Date.now() - timeBeforeProofsComputed) / 10) / 100 + 's ' +
                '(' + Math.trunc((Date.now() - timeBeforeProofsComputed) / (stop - start + 1) / 10) / 100 + 's per header)',
            );
        });
    }

    async submitBlock (block, blockNumber) {
        let sleepTimer = 1;
        const maxSleepTime = 10;
        const sleep = async () => {
            await new Promise((resolve, reject) => {
                setTimeout(resolve, sleepTimer * 1000);
            });
            if (sleepTimer < maxSleepTime) {
                sleepTimer += 1;
            }
        };
        let ok = false;
        for (let iters = 0; iters < 20; ++iters) {
            try {
                const last_block_number_onchain = (await this.ethClientContract.last_block_number()).toNumber();
                if (last_block_number_onchain > 0 && last_block_number_onchain < blockNumber - 1) {
                    console.log(`Sleeping ${sleepTimer} sec. The latest block on chain is ${last_block_number_onchain}, but need to submit block #${blockNumber}`);
                    await sleep();
                } else {
                    ok = true;
                    break;
                }
            } catch (e) {
                console.log('Block awaiting failed :(', e);
                await sleep();
            }
        }
        if (!ok) {
            process.exit(1);
        }

        // Check bridge state, may be changed since computation could be long
        const timeBeforeSubmission = Date.now();
        console.log(`Submitting block ${blockNumber} to EthClient`);
        const h512s = block.elements
            .filter((_, index) => index % 2 === 0)
            .map((element, index) => {
                return this.web3.utils.padLeft(element, 64) + this.web3.utils.padLeft(block.elements[index * 2 + 1], 64).substr(2);
            });

        const args = {
            block_header: this.web3.utils.hexToBytes(block.header_rlp),
            dag_nodes: h512s
                .filter((_, index) => index % 2 === 0)
                .map((element, index) => {
                    return {
                        dag_nodes: [element, h512s[index * 2 + 1]],
                        proof: block.merkle_proofs.slice(
                            index * block.proof_length,
                            (index + 1) * block.proof_length,
                        ).map(leaf => this.web3.utils.padLeft(leaf, 32)),
                    };
                }),
        };

        for (let i = 0; i < 10; ++i) {
            try {
                await this.ethClientContract.add_block_header(args, new BN('300000000000000'));
                console.log(
                    'Blocks submission took ' + Math.trunc((Date.now() - timeBeforeSubmission) / 10) / 100 + 's ' +
                    '(' + Math.trunc((Date.now() - timeBeforeSubmission) / 10) / 100 + 's per header)',
                );
                console.log(`Successfully submitted block ${blockNumber} to EthClient`);
                return;
            } catch (e) {
                // failed
                console.log(`Sleeping 0.5sec. Failed at iteration #${i}:`, e);
                await new Promise((resolve, reject) => {
                    setTimeout(resolve, 500);
                });
            }
        }
        throw new Error('Failed to submit a block');
    }

    async subscribeOnBlocksRangesFrom (block_number, handler) {
        let inBlocksCallbacks = false;
        let last_block_number = block_number;

        this.web3.eth.subscribe('newBlockHeaders', async (error, event) => {
            if (error) {
                console.log(error);
                return;
            }

            if (!inBlocksCallbacks && event.number - last_block_number > 4) {
                inBlocksCallbacks = true;

                const start = last_block_number;
                const stop = event.number - 2;
                last_block_number = stop;
                await handler(start, stop);

                inBlocksCallbacks = false;
            }
        });
    }
}

exports.Eth2NearRelay = Eth2NearRelay;
exports.web3BlockToRlp = web3BlockToRlp;
exports.execute = execute;
