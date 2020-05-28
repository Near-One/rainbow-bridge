const utils = require('ethereumjs-util');
const Web3 = require('web3');
const BN = require('bn.js');
const { EthProofExtractor, receiptFromWeb3, logFromWeb3 } = require('../eth-proof-extractor');
const { borshSchema } = require('../eth-prover-contract');
const { serialize } = require('../borsh');

// Tests EthProver in the following way:
// * Gets the last block submitted to EthClient;
// * Waits until EthClient accumulates 10 blocks;
// * For each block, for each transaction in this block, and for each log in that transaction, creates
//   a proof and sends it to EthProver.
class EthProverTester {
    constructor(ethNodeURL, ethClientContract, ethProverContract) {
        this.web3 = new Web3(ethNodeURL);
        this.ethNodeURL = ethNodeURL;
        this.ethClientContract = ethClientContract;
        this.ethProverContract = ethProverContract;
    }

    async run() {
        // Number of blocks to process.
        const numBlocks = 1;
        let firstBlock = (await this.ethClientContract.last_block_number()).toNumber() - 1;
        // let firstBlock = 10;
        let lastBlock = firstBlock;
        // Wait for the blocks to be accepted by the EthClient.
        while (firstBlock + numBlocks > lastBlock) {
            await sleep(1000);
            lastBlock = (await this.ethClientContract.last_block_number()).toNumber();
        }
        console.log(`firstBlock=${firstBlock}; lastBlock=${lastBlock}`);

        // Loop over blocks.
        let extractor = new EthProofExtractor();
        extractor.initialize(this.ethNodeURL);
        let test_num = 0;

        for (let currBlock = firstBlock; currBlock <= lastBlock; currBlock++) {
            let block = await this.web3.eth.getBlock(currBlock);
            for (let txHash of block.transactions) {
                const receipt = await extractor.extractReceipt(txHash);
                if (receipt.logs.length == 0) {
                    continue;
                }
                const block = await extractor.extractBlock(receipt.blockNumber);
                const tree = await extractor.buildTrie(block);
                const proof = await extractor.extractProof(block, tree, receipt.transactionIndex);
                let log_index = -1;
                for (const log of receipt.logs) {
                    log_index++;
                    console.log("===========================================================================");
                    console.log(`BLOCK NUMBER ${receipt.blockNumber}`);
                    console.log(`TX_HASH ${txHash}`);
                    console.log(`LOG_INDEX ${log_index}`);

                    // For debugging.
                    // let log_index = log.logIndex;
                    // console.log(`let log_index = ${log_index};`)
                    // console.log('let receipt_index = ' + proof.txIndex + ';');
                    // console.log('let header_data = Vec::from_hex("' + proof.header.serialize().toString('hex') + '").unwrap();');
                    // console.log('let receipt_data = Vec::from_hex("' + receiptFromWeb3(receipt).serialize().toString('hex') + '").unwrap();');
                    // console.log('let log_entry = Vec::from_hex("' + logFromWeb3(log).serialize().toString('hex') +'").unwrap();');
                    // console.log('let proof = vec![');
                    // for (let rec of proof.receiptProof) {
                    //     console.log('    vec![');
                    //     for (let r of rec) {
                    //         console.log('        Vec::from_hex("' + r.toString('hex') + '").unwrap(),');
                    //     }
                    //     console.log('    ],');
                    // }
                    // console.log('];');

                    const log_entry_data = logFromWeb3(log).serialize();
                    const receipt_index = proof.txIndex;
                    const receipt_data = receiptFromWeb3(receipt).serialize();
                    const header_data = proof.header.serialize();
                    let _proof = [];
                    for (let node of proof.receiptProof) {
                        _proof.push(utils.rlp.encode(node));
                    }

                    const skip_bridge_call = false;

                    // const borsh_log_index = serialize(borshSchema, 'u64', log_index).toString('hex');
                    // const borsh_log_entry_data = serialize(borshSchema, ['u8'], log_entry_data).toString('hex');
                    // const borsh_receipt_index = serialize(borshSchema, 'u64', receipt_index).toString('hex');
                    // const borsh_receipt_data = serialize(borshSchema, ['u8'], receipt_data).toString('hex');
                    // const borsh_header_data = serialize(borshSchema, ['u8'], header_data).toString('hex');
                    // const borsh_proof = serialize(borshSchema, [['u8']], _proof).toString('hex');
                    // const borsh_skip_bridge_call = serialize(borshSchema, 'bool', skip_bridge_call).toString('hex');
                    // console.log(`let actual_borsh_log_index = "${borsh_log_index}";`);
                    // console.log(`let actual_borsh_log_entry_data = "${borsh_log_entry_data}";`);
                    // console.log(`let actual_borsh_receipt_index = "${borsh_receipt_index}";`);
                    // console.log(`let actual_borsh_receipt_data = "${borsh_receipt_data}";`);
                    // console.log(`let actual_borsh_header_data = "${borsh_header_data}";`);
                    // console.log(`let actual_borsh_proof = "${borsh_proof}";`);
                    // console.log(`let actual_borsh_skip_bridge_call = "${borsh_skip_bridge_call}";`);


                    const args = {
                        log_index: log_index,
                        log_entry_data: log_entry_data,
                        receipt_index: receipt_index,
                        receipt_data: receipt_data,
                        header_data: header_data,
                        proof: _proof,
                        skip_bridge_call: skip_bridge_call,
                    };

                    // const borsh_args = serialize(borshSchema, 'verifyLogEntry', args).toString('hex');

                    let result = await this.ethProverContract.verify_log_entry(
                        args,
                        new BN('1000000000000000')
                    );

                    if (!result) {
                        process.exit(1);
                    }

                    test_num++;
                    console.log(`TEST NUM ${test_num} SUCCEEDED`);
                }
            }
        }
        extractor.destroy();
    }

    destroy() {
        if (this.web3.currentProvider.connection.close) { // Only WebSocket provider has close, HTTPS don't
            this.web3.currentProvider.connection.close();
        }
    }
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

exports.EthProverTester = EthProverTester;