const Web3 = require('rainbow-bridge-utils')
const Path = require('path')
const fs = require('fs').promises
const {
  EthProofExtractor,
  ethashproof,
  web3BlockToRlp,
  receiptFromWeb3,
  logFromWeb3
} = require('rainbow-bridge-eth2near-block-relay')
const utils = require('ethereumjs-util')

class ETHDump {
  static async execute (kindOfData, { path, startBlock, endBlock, ethNodeUrl }) {
    const web3 = new Web3(ethNodeUrl)
    const extractor = new EthProofExtractor()
    extractor.initialize(ethNodeUrl)

    if (kindOfData !== 'headers' && kindOfData !== 'proofs') {
      console.log(
        'Usage: node index.js eth-dump headers\n       node index.js eth-dump proofs'
      )
      process.exit(2)
    }
    if (!endBlock) {
      endBlock = await web3.eth.getBlockNumber()
    } else {
      endBlock = Number(endBlock)
    }
    if (!startBlock) {
      startBlock = Math.max(0, Number(endBlock) - 43000 + 1)
    } else if (Number(startBlock) < 0) {
      startBlock = Math.max(0, Number(endBlock) + Number(startBlock) + 1)
    } else {
      startBlock = Number(startBlock)
    }
    console.log(
      `Downloading block ${endBlock} down to ${startBlock} to ${path}. ${
        endBlock - startBlock + 1
      } blocks in total.`
    )

    for (let b = endBlock; b >= startBlock; b--) {
      if (kindOfData === 'headers') {
        await ETHDump.dumpHeaders(web3, b, path)
      } else if (kindOfData === 'proofs') {
        await ETHDump.dumpProofs(web3, extractor, b, path)
      }
    }

    try {
      // Only WebSocket provider can close.
      web3.currentProvider.connection.close()
    } catch (e) {}
    extractor.destroy()
  }

  static async dumpHeaders (web3, b, path) {
    console.log(`Downloading block ${b}`)
    const blockRlp = web3.utils.bytesToHex(
      web3BlockToRlp(await web3.eth.getBlock(b))
    )
    console.log(`Processing block ${b}`)

    const unparsedBlock = await ethashproof(
      `./eth2near/ethashproof/cmd/relayer/relayer ${blockRlp} | sed -e '1,/Json output/d'`
    )
    const block = JSON.parse(unparsedBlock)
    await ETHDump.saveBlock(b, block, path)
  }

  static async dumpProofs (web3, extractor, b, path) {
    const block = await web3.eth.getBlock(b)
    for (const txHash of block.transactions) {
      const receipt = await extractor.extractReceipt(txHash)
      if (receipt.logs.length === 0) {
        continue
      }
      const block = await extractor.extractBlock(receipt.blockNumber)
      const tree = await extractor.buildTrie(block)
      const proof = await extractor.extractProof(
        web3,
        block,
        tree,
        receipt.transactionIndex
      )
      let logIndex = -1
      for (const log of receipt.logs) {
        logIndex++
        const receiptIndex = proof.txIndex

        console.log(
          '==========================================================================='
        )
        console.log(`BLOCK_NUMBER ${receipt.blockNumber}`)
        console.log(`RECEIPT_INDEX ${receiptIndex}`)
        console.log(`TX_HASH ${txHash}`)
        console.log(`LOG_INDEX ${logIndex}`)

        const logEntryData = logFromWeb3(log).serialize()
        const receiptData = receiptFromWeb3(receipt).serialize()
        const headerData = proof.header_rlp
        const _proof = []
        for (const node of proof.receiptProof) {
          _proof.push(utils.rlp.encode(node))
        }

        const skipBridgeCall = false

        const args = {
          log_index: logIndex,
          log_entry_data: logEntryData.toString('hex'),
          receipt_index: receiptIndex,
          receipt_data: receiptData.toString('hex'),
          header_data: headerData.toString('hex'),
          proof: _proof.map((p) => p.toString('hex')),
          skip_bridge_call: skipBridgeCall
        }

        const file = Path.join(path, `${b}_${receiptIndex}_${logIndex}.json`)
        await fs.writeFile(file, JSON.stringify(args))
      }
    }
  }

  static async saveBlock (i, block, path) {
    const file = Path.join(path, `${i}.json`)
    await fs.writeFile(file, JSON.stringify(block))
  }
}

exports.ETHDump = ETHDump
