const Web3 = require('web3');
const Tree = require('merkle-patricia-tree');
const utils = require('ethereumjs-util');
const {
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

function receiptFromWeb3 (result, state_root) {
    return new Receipt([
        toBuffer(result.status ? 0x1 : state_root),
        toBuffer(result.cumulativeGasUsed),
        toBuffer(result.logsBloom),
        result.logs.map(logFromWeb3),
    ]);
}

function logFromWeb3 (result) {
    return new Log([
        toBuffer(result.address),
        result.topics.map(toBuffer),
        toBuffer(result.data),
    ]);
}

class EthProofExtractor {
    initialize (ethNodeURL) {
        // @ts-ignore
        this.web3 = new Web3(ethNodeURL);
    }

    async extractReceipt (txHash) {
        return await this.web3.eth.getTransactionReceipt(txHash);
    }

    async extractBlock (blockNumber) {
        return await this.web3.eth.getBlock(blockNumber);
    }

    async buildTrie (block) {
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

    async extractProof (web3, block, tree, transactionIndex) {
        const [, , stack] = await promisfy(tree.findPath, tree)(encode(transactionIndex));

        const blockData = await web3.eth.getBlock(block.number);
        // Correctly compose and encode the header.
        const header = [
            blockData.parentHash,
            blockData.sha3Uncles,
            blockData.miner,
            blockData.stateRoot,
            blockData.transactionsRoot,
            blockData.receiptsRoot,
            blockData.logsBloom,
            blockData.difficulty == '0' ? '0x' : web3.utils.toHex(blockData.difficulty),
            web3.utils.toHex(blockData.number),
            web3.utils.toHex(blockData.gasLimit),
            web3.utils.toHex(blockData.gasUsed),
            web3.utils.toHex(blockData.timestamp),
            blockData.extraData,
            blockData.mixHash,
            blockData.nonce,
        ];

        return {
            header_rlp: utils.rlp.encode(header),
            receiptProof: Proof.fromStack(stack),
            txIndex: transactionIndex,
        };
    }

    destroy () {
        if (this.web3.currentProvider.connection.close) { // Only WebSocket provider has close, HTTPS don't
            this.web3.currentProvider.connection.close();
        }
    }
}

exports.EthProofExtractor = EthProofExtractor;
exports.receiptFromWeb3 = receiptFromWeb3;
exports.logFromWeb3 = logFromWeb3;
