const Tree = require('merkle-patricia-tree')
const { Header, Proof, Receipt, Log, Account } = require('eth-object')
const utils = require('ethereumjs-util')
const { encode } = require('eth-util-lite')
const { promisfy } = require('promisfy')
const { RobustWeb3, JSONreplacer } = require('rainbow-bridge-utils')

function receiptFromWeb3 (result) {
  return Receipt.fromWeb3(result)
}

function logFromWeb3 (result) {
  return Log.fromWeb3(result)
}

class EthProofExtractor {
  initialize (ethNodeURL) {
    this.robustWeb3 = new RobustWeb3(ethNodeURL)
    this.web3 = this.robustWeb3.web3
  }

  async extractReceipt (txHash) {
    return await this.robustWeb3.getTransactionReceipt(txHash)
  }

  async extractBlock (blockNumber) {
    return await this.robustWeb3.getBlock(blockNumber)
  }

  async extractStorageProof (contractAddress, slotKey, blockNumber) {
    const proofFromWeb3 = await this.web3.eth.getProof(contractAddress, [slotKey], blockNumber)
    const blockData = await this.web3.eth.getBlock(blockNumber)
    const header = Header.fromWeb3(blockData)
    proofFromWeb3.nonce = this.web3.utils.toHex(proofFromWeb3.nonce)
    proofFromWeb3.balance = this.web3.utils.toHex(proofFromWeb3.balance)
    const account = Account.fromRpc(proofFromWeb3)
    return {
      header_rlp: header.serialize(),
      account_rlp: account.serialize(),
      account_proof: proofFromWeb3.accountProof,
      storage_proof: proofFromWeb3.storageProof[0].proof
    }
  }

  async buildTrie (block) {
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

  async extractProof (web3, block, tree, transactionIndex) {
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
      txIndex: transactionIndex
    }
  }

  destroy () {
    if (this.web3.currentProvider.connection && this.web3.currentProvider.connection.close) {
      // Only WebSocket provider has close, HTTPS don't
      this.web3.currentProvider.connection.close()
    }
  }
}

async function ethToNearFindProof ({ lockedEventRaw, ethNodeUrl }) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  try {
    const lockedEvent = JSON.parse(lockedEventRaw) || {}
    const extractor = new EthProofExtractor()
    extractor.initialize(ethNodeUrl)

    const receipt = await extractor.extractReceipt(lockedEvent.transactionHash)
    const block = await extractor.extractBlock(receipt.blockNumber)
    const tree = await extractor.buildTrie(block)
    const extractedProof = await extractor.extractProof(
      web3,
      block,
      tree,
      receipt.transactionIndex
    )
    // destroy extractor here to close its web3 connection
    extractor.destroy()

    let txLogIndex = -1

    let logFound = false
    let log
    for (const receiptLog of receipt.logs) {
      txLogIndex++
      const blockLogIndex = receiptLog.logIndex
      if (blockLogIndex === lockedEvent.logIndex) {
        logFound = true
        log = receiptLog
        break
      }
    }
    if (logFound) {
      const logEntryData = logFromWeb3(log).serialize()
      const receiptIndex = extractedProof.txIndex
      const receiptData = receiptFromWeb3(receipt).serialize()
      const headerData = extractedProof.header_rlp
      const proof = []
      for (const node of extractedProof.receiptProof) {
        proof.push(utils.rlp.encode(node))
      }
      const proofLocker = {
        log_index: txLogIndex,
        log_entry_data: logEntryData,
        receipt_index: receiptIndex,
        receipt_data: receiptData,
        header_data: headerData,
        proof
      }
      console.log(JSON.stringify({ block_number: block.number, proof_locker: proofLocker }, JSONreplacer))
    } else {
      console.log(`Failed to find log for event ${lockedEventRaw}`)
    }
  } catch (error) {
    console.log('Failed', error.toString())
  }
  web3.currentProvider.connection.close()
}

async function ethToNearFindStorageProof ({ contractAddress, storageKey, blockNumber, ethNodeUrl }) {
  const robustWeb3 = new RobustWeb3(ethNodeUrl)
  const web3 = robustWeb3.web3
  try {
    const extractor = new EthProofExtractor()
    extractor.initialize(ethNodeUrl)

    const extractedProof = await extractor.extractStorageProof(contractAddress, storageKey, blockNumber)
    extractor.destroy()

    const proof = {
      contract_address: utils.stripHexPrefix(contractAddress.toLowerCase()),
      storage_key: utils.stripHexPrefix(storageKey),
      block_number: blockNumber,
      header_data: extractedProof.header_rlp.toString('hex'),
      account_proof: extractedProof.account_proof.map(x => utils.stripHexPrefix(x)),
      expected_account_state: extractedProof.account_rlp.toString('hex'),
      storage_key_hash: utils.stripHexPrefix(web3.utils.keccak256(storageKey)),
      storage_proof: extractedProof.storage_proof.map(x => utils.stripHexPrefix(x))
    }
    console.log(JSON.stringify(proof, JSONreplacer))
  } catch (error) {
    console.log('Failed', error.toString())
  }
}

EthProofExtractor.fromWeb3 = (web3) => {
  const extractor = new EthProofExtractor()
  extractor.robustWeb3 = new RobustWeb3(web3.currentProvider.host)
  extractor.web3 = web3
  return extractor
}

exports.EthProofExtractor = EthProofExtractor
exports.receiptFromWeb3 = receiptFromWeb3
exports.logFromWeb3 = logFromWeb3
exports.ethToNearFindProof = ethToNearFindProof
exports.ethToNearFindStorageProof = ethToNearFindStorageProof
