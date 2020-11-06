const path = require('path')
const exec = require('child_process').exec
const BN = require('bn.js')
const { RobustWeb3, sleep, txnStatus, nearAPI, RainbowConfig } = require('rainbow-bridge-utils')
const { web3BlockToRlp, EthOnNearClientContract } = require('./eth-on-near-client')
const { EthOnNearProverContract } = require('./eth-on-near-prover')
const { EthProofExtractor, logFromWeb3, receiptFromWeb3 } = require('./eth-proof-extractor')
const MAX_SUBMIT_BLOCK = 10
const BRIDGE_SRC_DIR = path.join(__dirname, '..', '..')

// TODO @frol use config
const LIBS_SOL_SRC_DIR = path.join(
  BRIDGE_SRC_DIR,
  'node_modules/rainbow-bridge-sol'
)
const LIBS_RS_SRC_DIR = path.join(
  BRIDGE_SRC_DIR,
  'node_modules/rainbow-bridge-rs'
)
const LIBS_TC_SRC_DIR = path.join(
  BRIDGE_SRC_DIR,
  'node_modules/rainbow-token-connector'
)

RainbowConfig.declareOption(
  'near-network-id',
  'The identifier of the NEAR network that the given NEAR node is expected to represent.'
)
RainbowConfig.declareOption('near-node-url', 'The URL of the NEAR node.')
RainbowConfig.declareOption('eth-node-url', 'The URL of the Ethereum node.')
RainbowConfig.declareOption(
  'near-master-account',
  'The account of the master account on NEAR blockchain that can be used to deploy and initialize the test contracts.' +
    ' This account will also own the initial supply of the fungible tokens.'
)
RainbowConfig.declareOption(
  'near-master-sk',
  'The secret key of the master account on NEAR blockchain.'
)
RainbowConfig.declareOption(
  'eth-master-sk',
  'The secret key of the master account on Ethereum blockchain.'
)
RainbowConfig.declareOption(
  'near-client-account',
  'The account of the Near Client contract that can be used to accept ETH headers.',
  'rainbow_bridge_eth_on_near_client'
)
RainbowConfig.declareOption(
  'near-client-sk',
  'The secret key of the Near Client account. If not specified will use master SK.'
)
RainbowConfig.declareOption(
  'near-client-contract-path',
  'The path to the Wasm file containing the Near Client contract.',
  path.join(LIBS_RS_SRC_DIR, 'res/eth_client.wasm')
)
RainbowConfig.declareOption(
  'near-client-init-balance',
  'The initial balance of Near Client contract in femtoNEAR.',
  '100000000000000000000000000'
)
RainbowConfig.declareOption(
  'near-client-validate-ethash',
  'Whether validate ethash of submitted eth block, should set to true on mainnet and false on PoA testnets',
  'true'
)
RainbowConfig.declareOption(
  'near-client-trusted-signer',
  'When non empty, deploy as trusted-signer mode where only tursted signer can submit blocks to client',
  ''
)
RainbowConfig.declareOption(
  'near-prover-account',
  'The account of the Near Prover contract that can be used to accept ETH headers.',
  'rainbow_bridge_eth_on_near_prover'
)
RainbowConfig.declareOption(
  'near-prover-sk',
  'The secret key of the Near Prover account. If not specified will use master SK.'
)
RainbowConfig.declareOption(
  'near-prover-contract-path',
  'The path to the Wasm file containing the Near Prover contract.',
  path.join(LIBS_RS_SRC_DIR, 'res/eth_prover.wasm')
)
RainbowConfig.declareOption(
  'near-prover-init-balance',
  'The initial balance of Near Prover contract in femtoNEAR.',
  '100000000000000000000000000'
)
RainbowConfig.declareOption(
  'daemon',
  'Whether the process should be launched as a daemon.',
  'true',
  true
)
RainbowConfig.declareOption(
  'core-src',
  'Path to the nearcore source. It will be downloaded if not provided.',
  ''
)
RainbowConfig.declareOption(
  'nearup-src',
  'Path to the nearup source. It will be downloaded if not provided.',
  ''
)
RainbowConfig.declareOption(
  'eth-gas-multiplier',
  'How many times more in Ethereum gas are we willing to overpay.',
  '1'
)

// User-specific arguments.
RainbowConfig.declareOption(
  'near-token-factory-account',
  'The account of the token factory contract that will be used to mint tokens locked on Ethereum.',
  'neartokenfactory'
)
RainbowConfig.declareOption(
  'near-token-factory-sk',
  'The secret key of the token factory account. If not specified will use master SK.'
)
RainbowConfig.declareOption(
  'near-token-factory-contract-path',
  'The path to the Wasm file containing the token factory contract.',
  path.join(LIBS_TC_SRC_DIR, 'res/bridge_token_factory.wasm')
)
RainbowConfig.declareOption(
  'near-token-factory-init-balance',
  'The initial balance of token factory contract in yoctoNEAR.',
  '1000000000000000000000000000'
)
RainbowConfig.declareOption(
  'eth-locker-address',
  'ETH address of the locker contract.'
)
RainbowConfig.declareOption(
  'eth-locker-abi-path',
  'Path to the .abi file defining Ethereum locker contract. This contract works in pair with mintable fungible token on NEAR blockchain.',
  path.join(LIBS_TC_SRC_DIR, 'res/BridgeTokenFactory.full.abi')
)
RainbowConfig.declareOption(
  'eth-locker-bin-path',
  'Path to the .bin file defining Ethereum locker contract. This contract works in pair with mintable fungible token on NEAR blockchain.',
  path.join(LIBS_TC_SRC_DIR, 'res/BridgeTokenFactory.full.bin')
)
RainbowConfig.declareOption(
  'eth-erc20-address',
  'ETH address of the ERC20 contract.'
)
RainbowConfig.declareOption(
  'eth-erc20-abi-path',
  'Path to the .abi file defining Ethereum ERC20 contract.',
  path.join(LIBS_TC_SRC_DIR, 'res/TToken.full.abi')
)
RainbowConfig.declareOption(
  'eth-erc20-bin-path',
  'Path to the .bin file defining Ethereum ERC20 contract.',
  path.join(LIBS_TC_SRC_DIR, 'res/TToken.full.bin')
)
RainbowConfig.declareOption(
  'eth-ed25519-address',
  'ETH address of the ED25519 contract.'
)
RainbowConfig.declareOption(
  'eth-ed25519-abi-path',
  'Path to the .abi file defining Ethereum ED25519 contract.',
  path.join(LIBS_SOL_SRC_DIR, 'nearbridge/dist/Ed25519.full.abi')
)
RainbowConfig.declareOption(
  'eth-ed25519-bin-path',
  'Path to the .bin file defining Ethereum ED25519 contract.',
  path.join(LIBS_SOL_SRC_DIR, 'nearbridge/dist/Ed25519.full.bin')
)
RainbowConfig.declareOption(
  'eth-client-lock-eth-amount',
  'Amount of Ether that should be temporarily locked when submitting a new header to EthClient, in wei.',
  '100000000000000000000'
)
RainbowConfig.declareOption(
  'eth-client-lock-duration',
  'The challenge window during which anyone can challenge an incorrect ED25519 signature of the Near block, in EthClient, in seconds.',
  14400
)
RainbowConfig.declareOption(
  'eth-client-replace-duration',
  'Minimum time difference required to replace a block during challenge period, in EthClient, in seconds.',
  18000
)
RainbowConfig.declareOption(
  'eth-client-address',
  'ETH address of the EthClient contract.'
)
RainbowConfig.declareOption(
  'eth-client-abi-path',
  'Path to the .abi file defining Ethereum Client contract.',
  path.join(LIBS_SOL_SRC_DIR, 'nearbridge/dist/NearBridge.full.abi')
)
RainbowConfig.declareOption(
  'eth-client-bin-path',
  'Path to the .bin file defining Ethereum Client contract.',
  path.join(LIBS_SOL_SRC_DIR, 'nearbridge/dist/NearBridge.full.bin')
)
RainbowConfig.declareOption(
  'eth-prover-address',
  'ETH address of the EthProver contract.'
)
RainbowConfig.declareOption(
  'eth-prover-abi-path',
  'Path to the .abi file defining Ethereum Prover contract.',
  path.join(LIBS_SOL_SRC_DIR, 'nearprover/dist/NearProver.full.abi')
)
RainbowConfig.declareOption(
  'eth-prover-bin-path',
  'Path to the .bin file defining Ethereum Prover contract.',
  path.join(LIBS_SOL_SRC_DIR, 'nearprover/dist/NearProver.full.bin')
)
RainbowConfig.declareOption(
  'near2eth-relay-min-delay',
  "Minimum number of seconds to wait if the relay can't submit a block right away.",
  '1'
)
RainbowConfig.declareOption(
  'near2eth-relay-max-delay',
  "Maximum number of seconds to wait if the relay can't submit a block right away.",
  '600'
)
RainbowConfig.declareOption(
  'near2eth-relay-error-delay',
  'Number of seconds to wait before retrying if there is an error.',
  '1'
)
RainbowConfig.declareOption(
  'watchdog-delay',
  'Number of seconds to wait after validating all signatures.',
  '300'
)
RainbowConfig.declareOption(
  'watchdog-error-delay',
  'Number of seconds to wait before retrying if there is an error.',
  '1'
)
RainbowConfig.declareOption('near-erc20-account', 'Must be declared before set')

function ethashproof (command, _callback) {
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
  initialize (ethClientContract, ethNodeURL) {
    this.ethClientContract = ethClientContract
    // @ts-ignore
    this.robustWeb3 = new RobustWeb3(ethNodeURL)
    this.web3 = this.robustWeb3.web3
  }

  async run () {
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
          const blockPromises = []
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
          console.log(e)
        }
      } else {
        await sleep(10000)
      }
    }
  }

  async getParseBlock (blockNumber) {
    try {
      const blockRlp = this.web3.utils.bytesToHex(
        web3BlockToRlp(await this.robustWeb3.getBlock(blockNumber))
      )
      const unparsedBlock = await ethashproof(
        `${BRIDGE_SRC_DIR}/eth2near/ethashproof/cmd/relayer/relayer ${blockRlp} | sed -e '1,/Json output/d'`
      )
      // console.log('---')
      // console.log(unparsedBlock)
      return JSON.parse(unparsedBlock)
    } catch (e) {
      console.log(`Failed to get or parse block ${blockNumber}: ${e}`)
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

async function runEth2NearRelay () {
  const masterAccount = RainbowConfig.getParam('near-master-account')
  const masterSk = RainbowConfig.getParam('near-master-sk')
  const keyStore = new nearAPI.keyStores.InMemoryKeyStore()
  await keyStore.setKey(
    RainbowConfig.getParam('near-network-id'),
    masterAccount,
    nearAPI.KeyPair.fromString(masterSk)
  )
  const near = await nearAPI.connect({
    nodeUrl: RainbowConfig.getParam('near-node-url'),
    networkId: RainbowConfig.getParam('near-network-id'),
    masterAccount: masterAccount,
    deps: {
      keyStore: keyStore
    }
  })

  const relay = new Eth2NearRelay()
  const clientContract = new EthOnNearClientContract(
    new nearAPI.Account(near.connection, masterAccount),
    RainbowConfig.getParam('near-client-account')
  )
  await clientContract.accessKeyInit()
  console.log('Initializing eth2near-relay...')
  relay.initialize(clientContract, RainbowConfig.getParam('eth-node-url'))
  console.log('Starting eth2near-relay...')
  relay.run()
}

exports.Eth2NearRelay = Eth2NearRelay
exports.EthProofExtractor = EthProofExtractor
exports.ethashproof = ethashproof
exports.EthOnNearClientContract = EthOnNearClientContract
exports.EthOnNearProverContract = EthOnNearProverContract
exports.runEth2NearRelay = runEth2NearRelay
exports.logFromWeb3 = logFromWeb3
exports.receiptFromWeb3 = receiptFromWeb3

require('make-runnable')
