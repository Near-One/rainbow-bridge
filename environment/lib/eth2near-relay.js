const Web3 = require('web3');
const exec = require('child_process').exec;
const utils = require('ethereumjs-util');
const BN = require('bn.js');
const blockFromRpc = require('ethereumjs-block/from-rpc');
const { RobustWeb3, sleep } = require('../lib/robust');

function execute(command, _callback) {
    return new Promise(resolve => exec(command, (error, stdout, _stderr) => {
        if (error) {
            console.log(error);
        }
        resolve(stdout);
    }));
}

function web3BlockToRlp(blockData) {
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
    initialize(ethClientContract, ethNodeURL) {
        this.ethClientContract = ethClientContract;
        // @ts-ignore
        this.robustWeb3 = new RobustWeb3(ethNodeURL);
        this.web3 = this.robustWeb3.web3;
    }

    async run() {
        const robustWeb3 = this.robustWeb3;
        while (true) {
            let clientBlockNumber;
            let chainBlockNumber;
            try {
                // Even retry 10 times ethClientContract.last_block_number could still fail
                // Return back to loop to avoid crash eth-relay.
                clientBlockNumber = (await this.ethClientContract.last_block_number()).toNumber();
                console.log('Client block number is ' + clientBlockNumber);
                chainBlockNumber = await robustWeb3.getBlockNumber();
                console.log('Chain block number is ' + chainBlockNumber);
            } catch (e) {
                console.log(e);
                continue;
            }

            // Backtrack if chain switched the fork.
            while (true) {
                try {
                    const chainBlock = await robustWeb3.getBlock(clientBlockNumber);
                    const chainBlockHash = chainBlock.hash;
                    const clientHashes = await this.ethClientContract.known_hashes(clientBlockNumber);
                    if (clientHashes.find(x => x === chainBlockHash)) {
                        break;
                    } else {
                        console.log(`Block ${chainBlockHash} height: ${clientBlockNumber} is not known to the client. Backtracking.`);
                        clientBlockNumber -= 1;
                    }
                } catch (e) {
                    console.log(e)
                    continue;
                }
            }

            if (clientBlockNumber < chainBlockNumber) {
                try {
                    const blockRlp = this.web3.utils.bytesToHex(web3BlockToRlp(await this.robustWeb3.getBlock(clientBlockNumber + 1)));
                    const unparsedBlock = await execute(`./vendor/ethashproof/cmd/relayer/relayer ${blockRlp} | sed -e '1,/Json output/d'`);
                    const block = JSON.parse(unparsedBlock);
                    await this.submitBlock(block, clientBlockNumber + 1);
                } catch (e) {
                    console.log(`Failed to submit a block ${e}`);
                }
            } else {
                await sleep(10000);
            }
        }
    }

    async submitBlock(block, blockNumber) {
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

        console.log(`Submitting block ${blockNumber} to EthClient`);
        await this.ethClientContract.add_block_header(args, new BN('300000000000000'));
        console.log(`Successfully submitted block ${blockNumber} to EthClient`);
    }
}

exports.Eth2NearRelay = Eth2NearRelay;
exports.web3BlockToRlp = web3BlockToRlp;
exports.execute = execute;
