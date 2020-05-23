const Web3 = require('web3');
const Tree = require('merkle-patricia-tree');
const { Header, Proof, Receipt, Log } = require('eth-object');
const { encode, toBuffer } = require('eth-util-lite');
const { promisfy } = require('promisfy');

function receiptFromWeb3(result, state_root) {
    return new Receipt([
        toBuffer(result.status ? 0x1 : state_root),
        toBuffer(result.cumulativeGasUsed),
        toBuffer(result.logsBloom),
        result.logs.map(logFromWeb3)
    ]);
}

function logFromWeb3(result) {
    return new Log([
        toBuffer(result.address),
        result.topics.map(toBuffer),
        toBuffer(result.data)
    ]);
}

class EthProofExtractor {
    initialize(ethNodeURL) {
        this.web3 = new Web3(ethNodeURL);
    }

    async extractReceipt(txHash) {
        return await this.web3.eth.getTransactionReceipt(txHash);
    }

    async extractBlock(blockNumber) {
        return await this.web3.eth.getBlock(blockNumber);
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

    async extractProof(block, tree, transactionIndex) {
        const [_, __, stack] = await promisfy(tree.findPath, tree)(encode(transactionIndex));
        return {
            header: Header.fromRpc(block),
            receiptProof: Proof.fromStack(stack),
            txIndex: transactionIndex,
        };
    }

    // Print debug information for the given transaction.
    async debugPrint(txHash) {
        const receipt = await this.extractReceipt(txHash);
        console.log("RECEIPT %s", receipt);
        const block = await this.extractBlock(receipt.blockNumber);
        const tree = await this.buildTrie(block);
        const proof = await this.extractProof(block, tree, receipt.transactionIndex);

        console.log('let receipt_index = ' + proof.txIndex + ';');
        console.log('let header_data = Vec::from_hex("' + proof.header.serialize().toString('hex') + '").unwrap();');
        console.log('let receipt_data = Vec::from_hex("' + receiptFromWeb3(receipt).serialize().toString('hex') + '").unwrap();');
        let logs = '';
        for (const log of receipt.logs) {
            logs += 'Vec::from_hex("' + logFromWeb3(log).serialize().toString('hex') + '").unwrap(),'
        }
        console.log(`let logs = vec![ ${logs} ]`);
        console.log('let proof = vec![');
        for (let rec of proof.receiptProof) {
            console.log('    vec![');
            for (let r of rec) {
                console.log('        Vec::from_hex("' + r.toString('hex') + '").unwrap(),');
            }
            console.log('    ],');
        }
        console.log('];');
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
