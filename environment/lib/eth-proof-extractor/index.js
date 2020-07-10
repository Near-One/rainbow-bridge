const Web3 = require('web3');
const Tree = require('merkle-patricia-tree');
const utils = require('ethereumjs-util');
const {
    Header,
    Proof,
    Receipt,
    Log,
} = require('eth-object');
const {
    encode,
    toBuffer,
} = require('eth-util-lite');
const {
    promisfy,
} = require('promisfy');
const { web3GetBlockNumber, web3GetBlock } = require('../robust');

function receiptFromWeb3(result, state_root) {
    return new Receipt([
        toBuffer(result.status ? 0x1 : state_root),
        toBuffer(result.cumulativeGasUsed),
        toBuffer(result.logsBloom),
        result.logs.map(logFromWeb3),
    ]);
}

function logFromWeb3(result) {
    return new Log([
        toBuffer(result.address),
        result.topics.map(toBuffer),
        toBuffer(result.data),
    ]);
}

class EthProofExtractor {
    initialize(ethNodeURL) {
        // @ts-ignore
        this.web3 = new Web3(ethNodeURL);
    }

    async extractReceipt(txHash) {
        return await this.web3.eth.getTransactionReceipt(txHash);
    }

    async extractBlock(blockNumber) {
        return await web3GetBlock(this.web3, blockNumber);
    }

    async buildTrie(block) {
        const blockReceipts = await Promise.all(block.transactions.map(this.web3.eth.getTransactionReceipt));
        // Build a Patricia Merkle Trie
        const tree = new Tree();
        await Promise.all(blockReceipts.map(receipt => {
            const path = encode(receipt.transactionIndex);
            const serializedReceipt = receiptFromWeb3(receipt).serialize();
            return promisfy(tree.put, tree)(path, serializedReceipt);
        }));
        return tree;
    }

    async extractProof(web3, block, tree, transactionIndex) {
        const [, , stack] = await promisfy(tree.findPath, tree)(encode(transactionIndex));

        const blockData = await web3.eth.getBlock(block.number);
        // Correctly compose and encode the header.
        const header = Header.fromWeb3(blockData);
        return {
            header_rlp: header.serialize(),
            receiptProof: Proof.fromStack(stack),
            txIndex: transactionIndex,
        };
    }

    destroy() {
        if (this.web3.currentProvider.connection.close) { // Only WebSocket provider has close, HTTPS don't
            this.web3.currentProvider.connection.close();
        }
    }
}

exports.EthProofExtractor = EthProofExtractor;
exports.receiptFromWeb3 = receiptFromWeb3;
exports.logFromWeb3 = logFromWeb3;
