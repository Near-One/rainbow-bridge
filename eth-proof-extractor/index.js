const Tree = require('merkle-patricia-tree')
const { Header, Proof, Receipt, Log } = require('eth-object')
const { encode } = require('eth-util-lite')
const { promisfy } = require('promisfy')
const { RobustWeb3 } = require('../rainbow/robust')

function receiptFromWeb3(result) {
  return Receipt.fromWeb3(result)
}

function logFromWeb3(result) {
  return Log.fromWeb3(result)
}

class EthProofExtractor {
  initialize(ethNodeURL) {
    // @ts-ignore
    this.robustWeb3 = new RobustWeb3(ethNodeURL)
    this.web3 = this.robustWeb3.web3
  }

  async extractReceipt(txHash) {
    return await this.robustWeb3.getTransactionReceipt(txHash)
  }

  async extractBlock(blockNumber) {
    return await this.robustWeb3.getBlock(blockNumber)
  }

  async buildTrie(block) {
    const blockReceipts = await Promise.all(
      block.transactions.map((t) => this.robustWeb3.getTransactionReceipt(t))
    )
    // Build a Patricia Merkle Trie
    const tree = new Tree()
    await Promise.all(
      blockReceipts.map((receipt) => {
        const path = encode(receipt.transactionIndex)
        const serializedReceipt = receiptFromWeb3(receipt).serialize()
        return promisfy(tree.put, tree)(path, serializedReceipt)
      })
    )
    return tree
  }

  async extractProof(web3, block, tree, transactionIndex) {
    const [, , stack] = await promisfy(
      tree.findPath,
      tree
    )(encode(transactionIndex))

    const blockData = await web3.eth.getBlock(block.number)
    // Correctly compose and encode the header.
    const header = Header.fromWeb3(blockData)
    return {
      header_rlp: header.serialize(),
      receiptProof: Proof.fromStack(stack),
      txIndex: transactionIndex,
    }
  }

  destroy() {
    if (this.web3.currentProvider.connection.close) {
      // Only WebSocket provider has close, HTTPS don't
      this.web3.currentProvider.connection.close()
    }
  }
}

EthProofExtractor.fromWeb3 = (web3) => {
    let extractor = new EthProofExtractor();
    extractor.robustWeb3 = new RobustWeb3(web3.currentProvider.host);
    extractor.web3 = web3;
    return extractor;
}

exports.EthProofExtractor = EthProofExtractor
exports.receiptFromWeb3 = receiptFromWeb3
exports.logFromWeb3 = logFromWeb3
