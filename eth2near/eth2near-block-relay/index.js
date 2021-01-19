const path = require('path')
const exec = require('child_process').exec
const BN = require('bn.js')
const {
  RobustWeb3,
  sleep,
  txnStatus
} = require('rainbow-bridge-utils')
const {
  web3BlockToRlp,
  EthOnNearClientContract
} = require('./eth-on-near-client')
const {
  EthOnNearProverContract
} = require('./eth-on-near-prover')
const {
  EthProofExtractor,
  logFromWeb3,
  receiptFromWeb3,
  ethToNearFindProof
} = require('./eth-proof-extractor')

const {
  HttpPrometheus
} = require('../../utils/http-prometheus.js')

const BRIDGE_SRC_DIR = path.join(__dirname, '..', '..')

function ethashproof (command, _callback) {
  return new Promise((resolve) =>
    exec(command, (error, stdout, _stderr) => {
      if (error) {
        console.error(error)
      }
      resolve(stdout)
    })
  )
}

class Eth2NearRelay {
  initialize (ethClientContract, {
    totalSubmitBlock,
    ethNodeUrl,
    metricsPort
  }) {
    this.totalSubmitBlock = parseInt(totalSubmitBlock)
    this.ethClientContract = ethClientContract
    // @ts-ignore
    this.robustWeb3 = new RobustWeb3(ethNodeUrl)
    this.web3 = this.robustWeb3.web3
    this.metricsPort = metricsPort
  }

  async run () {
    const robustWeb3 = this.robustWeb3
    const httpPrometheus = new HttpPrometheus(this.metricsPort, 'near_bridge_eth2near_')

    const clientBlockNumberGauge = httpPrometheus.gauge('client_block_number', 'current client block number')
    const chainBlockNumberGauge = httpPrometheus.gauge('chain_block_number', 'current chain block number')

    while (true) {
      let clientBlockNumber
      let chainBlockNumber
      try {
        // Even retry 10 times ethClientContract.last_block_number could still fail
        // Return back to loop to avoid crash eth2near-relay.
        clientBlockNumber = (
          await this.ethClientContract.last_block_number()
        ).toNumber()
        clientBlockNumberGauge.set(clientBlockNumber)
        chainBlockNumber = await robustWeb3.getBlockNumber()
        chainBlockNumberGauge.set(chainBlockNumber)
      } catch (e) {
        console.error(e)
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
            clientBlockNumberGauge.set(clientBlockNumber)
          }
        } catch (e) {
          console.error(e)
          continue
        }
      }

      if (clientBlockNumber < chainBlockNumber) {
        try {
          console.log(clientBlockNumber, this.totalSubmitBlock, clientBlockNumber + this.totalSubmitBlock)
          // Submit add_block txns
          const blockPromises = []
          let endBlock = Math.min(
            clientBlockNumber + this.totalSubmitBlock,
            chainBlockNumber
          )
          if (clientBlockNumber < 5) {
            // Initially, do not add block concurrently
            endBlock = clientBlockNumber + 1
          }
          for (let i = clientBlockNumber + 1; i <= endBlock; i++) {
            blockPromises.push(this.getParseBlock(i))
          }
          const blocks = await Promise.all(blockPromises)
          console.log(
            `Got and parsed block ${clientBlockNumber + 1} to block ${endBlock}`
          )

          const txHashes = []
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
          console.error(e)
        }
      } else {
        await sleep(10000)
      }
    }
  }

  async getParseBlock (blockNumber) {
    try {
      const block = await this.robustWeb3.getBlock(blockNumber)
      const blockRlp = this.web3.utils.bytesToHex(
        web3BlockToRlp(block)
      )
      const unparsedBlock = await ethashproof(
        `${BRIDGE_SRC_DIR}/eth2near/ethashproof/cmd/relayer/relayer ${blockRlp} | sed -e '1,/Json output/d'`
      )
      return JSON.parse(unparsedBlock)
    } catch (e) {
      console.error(`Failed to get or parse block ${blockNumber}: ${e}`)
    }
  }

  async submitBlock (block, blockNumber) {
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
              .map((leaf) => this.web3.utils.padLeft(leaf, 32))
          }
        })
    }

    console.log(`Submitting block ${blockNumber} to EthClient`)
    return await this.ethClientContract.add_block_header_async(
      args,
      new BN('300000000000000')
    )
  }
}

exports.Eth2NearRelay = Eth2NearRelay
exports.EthProofExtractor = EthProofExtractor
exports.ethashproof = ethashproof
exports.EthOnNearClientContract = EthOnNearClientContract
exports.EthOnNearProverContract = EthOnNearProverContract
exports.logFromWeb3 = logFromWeb3
exports.receiptFromWeb3 = receiptFromWeb3
exports.ethToNearFindProof = ethToNearFindProof
