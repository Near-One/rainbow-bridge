const path = require('path')
const os = require('os')
const exec = require('child_process').exec
const utils = require('ethereumjs-util')
const BN = require('bn.js')
const { RobustWeb3, sleep } = require('rainbow-bridge-lib/rainbow/robust')
const { txnStatus } = require('rainbow-bridge-lib/rainbow/borsh')
const { web3BlockToRlp } = require('rainbow-bridge-lib/eth2near-relay')
const MAX_SUBMIT_BLOCK = 10
const BRIDGE_SRC_DIR = path.join(__dirname, '..')

function execute(command, _callback) {
  return new Promise((resolve) =>
    exec(command, (error, stdout, _stderr) => {
      if (error) {
        console.log(error)
      }
      resolve(stdout)
    })
  )
}

class Eth2NearRelay {
  initialize(ethClientContract, ethNodeURL) {
    this.ethClientContract = ethClientContract
    // @ts-ignore
    this.robustWeb3 = new RobustWeb3(ethNodeURL)
    this.web3 = this.robustWeb3.web3
  }

  async run() {
    const robustWeb3 = this.robustWeb3
    while (true) {
      let clientBlockNumber
      let chainBlockNumber
      try {
        // Even retry 10 times ethClientContract.last_block_number could still fail
        // Return back to loop to avoid crash eth2near-relay.
        clientBlockNumber = (
          await this.ethClientContract.last_block_number()
        ).toNumber()
        console.log('Client block number is ' + clientBlockNumber)
        chainBlockNumber = await robustWeb3.getBlockNumber()
        console.log('Chain block number is ' + chainBlockNumber)
      } catch (e) {
        console.log(e)
        continue
      }

      // Backtrack if chain switched the fork.
      while (true) {
        try {
          const chainBlock = await robustWeb3.getBlock(clientBlockNumber)
          const chainBlockHash = chainBlock.hash
          const clientHashes = await this.ethClientContract.known_hashes(
            clientBlockNumber
          )
          if (clientHashes.find((x) => x === chainBlockHash)) {
            break
          } else {
            console.log(
              `Block ${chainBlockHash} height: ${clientBlockNumber} is not known to the client. Backtracking.`
            )
            clientBlockNumber -= 1
          }
        } catch (e) {
          console.log(e)
          continue
        }
      }

      if (clientBlockNumber < chainBlockNumber) {
        try {
          // Submit add_block txns
          let blockPromises = []
          let endBlock = Math.min(
            clientBlockNumber + MAX_SUBMIT_BLOCK,
            chainBlockNumber
          )
          if (clientBlockNumber < 5) {
            // Initially, do not add block concurrently
            endBlock = clientBlockNumber + 1
          }
          for (let i = clientBlockNumber + 1; i <= endBlock; i++) {
            blockPromises.push(this.getParseBlock(i))
          }
          let blocks = await Promise.all(blockPromises)
          console.log(
            `Got and parsed block ${clientBlockNumber + 1} to block ${endBlock}`
          )

          let txHashes = []
          for (let i = clientBlockNumber + 1, j = 0; i <= endBlock; i++, j++) {
            txHashes.push(await this.submitBlock(blocks[j], i))
          }

          console.log(
            `Submit txn to add block ${
              clientBlockNumber + 1
            } to block ${endBlock}`
          )

          // Wait add_block txns commit
          await Promise.all(
            txHashes.map((txHash) =>
              txnStatus(this.ethClientContract.account, txHash, 10, 2000)
            )
          )
          console.log(
            `Success added block ${clientBlockNumber + 1} to block ${endBlock}`
          )
        } catch (e) {
          console.log(e)
        }
      } else {
        await sleep(10000)
      }
    }
  }

  async getParseBlock(blockNumber) {
    try {
      const blockRlp = this.web3.utils.bytesToHex(
        web3BlockToRlp(await this.robustWeb3.getBlock(blockNumber))
      )
      const unparsedBlock = await execute(
        `${BRIDGE_SRC_DIR}/vendor/ethashproof/cmd/relayer/relayer ${blockRlp} | sed -e '1,/Json output/d'`
      )
      console.log('---')
      console.log(unparsedBlock)
      return JSON.parse(unparsedBlock)
    } catch (e) {
      console.log(`Failed to get or parse block ${blockNumber}: ${e}`)
    }
  }

  async submitBlock(block, blockNumber) {
    const h512s = block.elements
      .filter((_, index) => index % 2 === 0)
      .map((element, index) => {
        return (
          this.web3.utils.padLeft(element, 64) +
          this.web3.utils.padLeft(block.elements[index * 2 + 1], 64).substr(2)
        )
      })

    const args = {
      block_header: this.web3.utils.hexToBytes(block.header_rlp),
      dag_nodes: h512s
        .filter((_, index) => index % 2 === 0)
        .map((element, index) => {
          return {
            dag_nodes: [element, h512s[index * 2 + 1]],
            proof: block.merkle_proofs
              .slice(
                index * block.proof_length,
                (index + 1) * block.proof_length
              )
              .map((leaf) => this.web3.utils.padLeft(leaf, 32)),
          }
        }),
    }

    console.log(`Submitting block ${blockNumber} to EthClient`)
    return await this.ethClientContract.add_block_header_async(
      args,
      new BN('300000000000000')
    )
  }
}

exports.Eth2NearRelay = Eth2NearRelay
exports.execute = execute
