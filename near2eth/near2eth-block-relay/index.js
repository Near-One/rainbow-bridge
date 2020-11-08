const fs = require('fs')
const path = require('path')
// @ts-ignore
const bs58 = require('bs58')
// @ts-ignore
const { toBuffer } = require('eth-util-lite')
const { BN } = require('ethereumjs-util')
const {
  RainbowConfig,
  sleep,
  RobustWeb3,
  normalizeEthKey,
  borshify,
  borshifyInitialValidators,
  nearAPI
} = require('rainbow-bridge-utils')

// TODO @frol use config
const BRIDGE_SRC_DIR = __dirname
const LIBS_SOL_SRC_DIR = path.join(
  BRIDGE_SRC_DIR,
  '../../node_modules/rainbow-bridge-sol'
)
const LIBS_RS_SRC_DIR = path.join(
  BRIDGE_SRC_DIR,
  '../../node_modules/rainbow-bridge-rs'
)
const LIBS_TC_SRC_DIR = path.join(
  BRIDGE_SRC_DIR,
  '../../node_modules/rainbow-token-connector'
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

class Near2EthRelay {
  async initialize (ethMasterSk = null) {
    // @ts-ignore
    this.robustWeb3 = new RobustWeb3(RainbowConfig.getParam('eth-node-url'))
    this.web3 = this.robustWeb3.web3
    this.ethMasterAccount = this.web3.eth.accounts.privateKeyToAccount(
      ethMasterSk
        ? normalizeEthKey(ethMasterSk)
        : normalizeEthKey(RainbowConfig.getParam('eth-master-sk'))
    )
    this.web3.eth.accounts.wallet.add(this.ethMasterAccount)
    this.web3.eth.defaultAccount = this.ethMasterAccount.address
    this.ethMasterAccount = this.ethMasterAccount.address

    const keyStore = new nearAPI.keyStores.InMemoryKeyStore()
    this.near = await nearAPI.connect({
      nodeUrl: RainbowConfig.getParam('near-node-url'),
      networkId: RainbowConfig.getParam('near-network-id'),
      deps: {
        keyStore: keyStore
      }
    })

    // Declare Near2EthClient contract.
    this.clientContract = new this.web3.eth.Contract(
      // @ts-ignore
      JSON.parse(
        fs.readFileSync(RainbowConfig.getParam('eth-client-abi-path'))
      ),
      RainbowConfig.getParam('eth-client-address'),
      {
        from: this.ethMasterAccount,
        handleRevert: true
      }
    )

    // Check if initialization is needed.
    try {
      console.log('Checking whether client is initialized.')
      const isInitialized = await this.clientContract.methods
        .initialized()
        .call()
      if (!isInitialized) {
        console.log('Client is not initialized. Initializing.')
        // Get most recent block from Near blockchain.
        const status = await this.near.connection.provider.status()
        // Get the block two blocks before that, to make sure it is final.
        const headBlock = await this.near.connection.provider.block({
          blockId: status.sync_info.latest_block_height
        })
        // @ts-ignore
        const lastFinalBlockHash = headBlock.header.last_final_block
        // The finalized block is not immediately available so we wait for it to become available.
        let lightClientBlock = null
        let currentValidators = null
        while (!lightClientBlock) {
          // @ts-ignore
          currentValidators = await this.near.connection.provider.sendJsonRpc(
            'EXPERIMENTAL_validators_ordered',
            [lastFinalBlockHash]
          )
          if (!currentValidators) {
            await sleep(300)
            continue
          }
          lightClientBlock = await this.near.connection.provider.sendJsonRpc(
            'next_light_client_block',
            [lastFinalBlockHash]
          )
          if (!lightClientBlock) {
            await sleep(300)
            continue
          }
        }
        console.log('Initializing with validators')
        console.log(`${JSON.stringify(currentValidators)}`)
        const borshInitialValidators = borshifyInitialValidators(
          currentValidators
        )
        // @ts-ignore
        let gasPrice = new BN(await this.web3.eth.getGasPrice()).mul(
          new BN(RainbowConfig.getParam('eth-gas-multiplier'))
        )
        let err
        for (let i = 0; i < 10; i++) {
          try {
            await this.clientContract.methods
              .initWithValidators(borshInitialValidators)
              .send({
                from: this.ethMasterAccount,
                gas: 4000000,
                handleRevert: true,
                gasPrice
              })
          } catch (e) {
            if (e.message.includes('replacement transaction underpriced')) {
              gasPrice = gasPrice.mul(new BN(11)).div(new BN(10))
              continue
            }
            err = e
          }
          break
        }
        if (err) {
          console.log('Failure')
          console.log(err)
          process.exit(1)
        }

        console.log('Initializing with block')
        console.log(`${JSON.stringify(lightClientBlock)}`)
        const borshBlock = borshify(lightClientBlock)
        for (let i = 0; i < 10; i++) {
          try {
            await this.clientContract.methods.initWithBlock(borshBlock).send({
              from: this.ethMasterAccount,
              gas: 4000000,
              handleRevert: true,
              gasPrice: new BN(await this.web3.eth.getGasPrice()).mul(
                new BN(RainbowConfig.getParam('eth-gas-multiplier'))
              )
            })
          } catch (e) {
            if (e.message.includes('replacement transaction underpriced')) {
              gasPrice = gasPrice.mul(new BN(11)).div(new BN(10))
              continue
            }
            err = e
          }
          break
        }
        if (err) {
          console.log('Failure')
          console.log(err)
          process.exit(1)
        }
      }
      console.log('Client is initialized.')
    } catch (txRevertMessage) {
      console.log('Failure.')
      console.log(txRevertMessage.toString())
      process.exit(1)
    }
  }

  async runInternal (submitInvalidBlock) {
    const clientContract = this.clientContract
    const robustWeb3 = this.robustWeb3
    const near = this.near
    const ethMasterAccount = this.ethMasterAccount
    const web3 = this.web3

    const minDelay = Number(RainbowConfig.getParam('near2eth-relay-min-delay'))
    const maxDelay = Number(RainbowConfig.getParam('near2eth-relay-max-delay'))
    const errorDelay = Number(
      RainbowConfig.getParam('near2eth-relay-error-delay')
    )

    while (true) {
      try {
        // Determine the next action: sleep or attempt an update.
        const bridgeState = await clientContract.methods.bridgeState().call()
        const currentBlockHash = toBuffer(
          await clientContract.methods
            .blockHashes(bridgeState.currentHeight)
            .call()
        )
        const lastBlock = await near.connection.provider.sendJsonRpc(
          'next_light_client_block',
          [bs58.encode(currentBlockHash)]
        )
        const replaceDuration = web3.utils.toBN(
          await clientContract.methods.replaceDuration().call()
        )
        const nextValidAt = web3.utils.toBN(bridgeState.nextValidAt)
        let replaceDelay
        if (!nextValidAt.isZero()) {
          replaceDelay = web3.utils
            .toBN(bridgeState.nextTimestamp)
            .add(replaceDuration)
            .sub(web3.utils.toBN(lastBlock.inner_lite.timestamp))
        }
        // console.log({bridgeState, currentBlockHash, lastBlock, replaceDuration}) // DEBUG
        if (bridgeState.currentHeight < lastBlock.inner_lite.height) {
          if (nextValidAt.isZero() || replaceDelay.cmpn(0) <= 0) {
            console.log(
              `Trying to submit new block at height ${lastBlock.inner_lite.height}.`
            )

            // Check whether master account has enough balance at stake.
            const lockEthAmount = await clientContract.methods
              .lockEthAmount()
              .call()
            const balance = await clientContract.methods
              .balanceOf(ethMasterAccount)
              .call()
            if (balance === '0') {
              console.log(
                `The sender account does not have enough stake. Transferring ${lockEthAmount} wei.`
              )
              await clientContract.methods.deposit().send({
                from: ethMasterAccount,
                gas: 1000000,
                handleRevert: true,
                value: new BN(lockEthAmount),
                gasPrice: new BN(await web3.eth.getGasPrice()).mul(
                  new BN(RainbowConfig.getParam('eth-gas-multiplier'))
                )
              })
              console.log('Transferred.')
            }

            const borshBlock = borshify(lastBlock)
            if (submitInvalidBlock) {
              console.log('Mutate block by one byte')
              console.log(borshBlock)
              borshBlock[Math.floor(borshBlock.length * Math.random())] += 1
            }
            await clientContract.methods.addLightClientBlock(borshBlock).send({
              from: ethMasterAccount,
              gas: 4000000,
              handleRevert: true,
              gasPrice: new BN(await web3.eth.getGasPrice()).mul(
                new BN(RainbowConfig.getParam('eth-gas-multiplier'))
              )
            })

            if (submitInvalidBlock) {
              console.log('Successfully submit invalid block')
              return process.exit(0)
            }
            console.log('Submitted.')
            continue
          }
        }
        // Going to sleep, compute the delay.
        let delay = maxDelay
        if (!nextValidAt.isZero()) {
          const latestBlock = await robustWeb3.getBlock('latest')
          delay = Math.min(
            delay,
            nextValidAt.toNumber() - latestBlock.timestamp
          )
          delay = Math.min(
            delay,
            replaceDelay.div(new web3.utils.BN(1e9)).toNumber()
          )
        }
        delay = Math.max(delay, minDelay)
        console.log(
          `Client height is ${bridgeState.currentHeight}, chain height is ${lastBlock.inner_lite.height}. Sleeping for ${delay} seconds.`
        )
        await sleep(1000 * delay)
      } catch (e) {
        console.log('Error', e)
        await sleep(1000 * errorDelay)
      }
    }
  }

  DANGERsubmitInvalidNearBlock () {
    return this.runInternal(true)
  }

  run () {
    return this.runInternal(false)
  }
}

async function runNear2EthRelay (ethMasterSk) {
  const relay = new Near2EthRelay()
  await relay.initialize(ethMasterSk)
  relay.run()
}

exports.Near2EthRelay = Near2EthRelay
exports.borshify = borshify
exports.runNear2EthRelay = runNear2EthRelay

require('make-runnable')
