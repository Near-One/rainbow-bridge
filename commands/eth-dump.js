const Web3 = require('web3')
const Path = require('path')
const fs = require('fs').promises
const { web3BlockToRlp } = require('rainbow-bridge-lib/eth2near-relay')
const { execute } = require('../lib/eth2near-relay')
const {
  EthProofExtractor,
  receiptFromWeb3,
  logFromWeb3,
} = require('rainbow-bridge-lib/eth-proof-extractor')
const utils = require('ethereumjs-util')

class ETHDump {
  static async execute(kindOfData, { path, startBlock, endBlock, ethNodeUrl }) {
    // @ts-ignore
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

  static async dumpHeaders(web3, b, path) {
    console.log(`Downloading block ${b}`)
    const blockRlp = web3.utils.bytesToHex(
      web3BlockToRlp(await web3.eth.getBlock(b))
    )
    console.log(`Processing block ${b}`)

    const unparsedBlock = await execute(
      `./vendor/ethashproof/cmd/relayer/relayer ${blockRlp} | sed -e '1,/Json output/d'`
    )
    const block = JSON.parse(unparsedBlock)
    await ETHDump.saveBlock(b, block, path)
  }

  static async dumpProofs(web3, extractor, b, path) {
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
      let log_index = -1
      for (const log of receipt.logs) {
        log_index++
        const receipt_index = proof.txIndex

        console.log(
          '==========================================================================='
        )
        console.log(`BLOCK NUMBER ${receipt.blockNumber}`)
        console.log(`RECEIPT_INDEX ${receipt_index}`)
        console.log(`TX_HASH ${txHash}`)
        console.log(`LOG_INDEX ${log_index}`)

        const log_entry_data = logFromWeb3(log).serialize()
        const receipt_data = receiptFromWeb3(receipt).serialize()
        const header_data = proof.header_rlp
        const _proof = []
        for (const node of proof.receiptProof) {
          _proof.push(utils.rlp.encode(node))
        }

        const skip_bridge_call = false

        const args = {
          log_index: log_index,
          log_entry_data: log_entry_data.toString('hex'),
          receipt_index: receipt_index,
          receipt_data: receipt_data.toString('hex'),
          header_data: header_data.toString('hex'),
          proof: _proof.map((p) => p.toString('hex')),
          skip_bridge_call: skip_bridge_call,
        }

        const file = Path.join(path, `${b}_${receipt_index}_${log_index}.json`)
        await fs.writeFile(file, JSON.stringify(args))
      }
    }
  }

  static async saveBlock(i, block, path) {
    const file = Path.join(path, `${i}.json`)
    await fs.writeFile(file, JSON.stringify(block))
  }
}

exports.ETHDump = ETHDump
