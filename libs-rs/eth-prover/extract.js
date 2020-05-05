const Web3 = require('web3');
const Tree = require('merkle-patricia-tree');
const { Header, Proof, Receipt, Log } = require('eth-object');
const { encode, toBuffer } = require('eth-util-lite');
const { promisfy } = require('promisfy');

function receiptFromWeb3(result, state_root) {
    return new Receipt([
        toBuffer(result.status ? 0x1 : 0x0),
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

// NODE_URL="https://mainnet.infura.io/v3/b5f870422ee5454fb11937e947154cd2" TX_HASH="0xb540248a9cca048c5861dec953d7a776bc1944319b9bd27a462469c8a437f4ff" EVENT_INDEX=0 node extract.js
// NODE_URL="https://mainnet.infura.io/v3/b5f870422ee5454fb11937e947154cd2" TX_HASH="0xa7e1633e8099ea2b72496207b76a0e04a761c52f48c82bfcf6b327495258e4e0" EVENT_INDEX=0 node extract.js

// https://github.com/zmitton/eth-proof/blob/master/getProof.js#L39
(async function () {
    const web3 = new Web3(process.env.NODE_URL);
    
    const targetReceipt = await web3.eth.getTransactionReceipt(process.env.TX_HASH);
    const event = targetReceipt.logs[process.env.EVENT_INDEX];
    const block = await web3.eth.getBlock(targetReceipt.blockNumber);
    const blockReceipts = await Promise.all(block.transactions.map(web3.eth.getTransactionReceipt));
    
    // Build a Patricia Merkle Trie
    const tree = new Tree();
    await Promise.all(blockReceipts.map((receipt, index) => {
        const path = encode(index);
        const serializedReceipt = receiptFromWeb3(receipt).serialize();
        return promisfy(tree.put, tree)(path, serializedReceipt);
    }));

    // Extract Proof
    const [_, __, stack] = await promisfy(tree.findPath, tree)(encode(targetReceipt.transactionIndex));
    const proof = {
        header: Header.fromRpc(block),
        receiptProof: Proof.fromStack(stack),
        txIndex: targetReceipt.transactionIndex,
    };

    // Print proof in Rust syntax
    console.log('let log_index = ' + process.env.EVENT_INDEX + ';');
    console.log('let receipt_index = ' + proof.txIndex + ';');
    console.log('let header_data = Vec::from_hex("' + proof.header.serialize().toString('hex') + '").unwrap();');
    console.log('let receipt_data = Vec::from_hex("' + receiptFromWeb3(blockReceipts[proof.txIndex]).serialize().toString('hex') + '").unwrap();');
    console.log('let log_entry = Vec::from_hex("' + logFromWeb3(event).serialize().toString('hex') + '").unwrap();');
    console.log('let proof = vec![');
    for (let rec of proof.receiptProof) {
        console.log('    vec![');
        for (let r of rec) {
            console.log('        Vec::from_hex("' + r.toString('hex') + '").unwrap(),');
        }
        console.log('    ],');
    }
    console.log('];');
})()
