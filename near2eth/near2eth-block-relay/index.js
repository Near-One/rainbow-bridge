const fs = require('fs')
const bs58 = require('bs58')
const { toBuffer } = require('eth-util-lite')
const { BN } = require('ethereumjs-util')
const lodash = require('lodash')
const {
  sleep,
  RobustWeb3,
  normalizeEthKey,
  borshify,
  borshifyInitialValidators,
  nearAPI
} = require('rainbow-bridge-utils')
const { HttpPrometheus } = require('../../utils/http-prometheus.js')

class Near2EthRelay {
  async initialize ({
    nearNodeUrl,
    nearNetworkId,
    ethNodeUrl,
    ethMasterSk,
    ethClientArtifactPath,
    ethClientAddress,
    ethGasMultiplier,
    metricsPort
  }) {
    this.robustWeb3 = new RobustWeb3(ethNodeUrl)
    this.web3 = this.robustWeb3.web3
    this.ethMasterAccount = this.web3.eth.accounts.privateKeyToAccount(
      normalizeEthKey(ethMasterSk)
    )
    this.web3.eth.accounts.wallet.add(this.ethMasterAccount)
    this.web3.eth.defaultAccount = this.ethMasterAccount.address
    this.ethMasterAccount = this.ethMasterAccount.address
    this.metricsPort = metricsPort

    const keyStore = new nearAPI.keyStores.InMemoryKeyStore()
    this.near = await nearAPI.connect({
      nodeUrl: nearNodeUrl,
      networkId: nearNetworkId,
      deps: {
        keyStore: keyStore
      }
    })

    // Declare Near2EthClient contract.
    this.clientContract = new this.web3.eth.Contract(
      JSON.parse(fs.readFileSync(ethClientArtifactPath)).abi,
      ethClientAddress,
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
        const lastFinalBlockHash = headBlock.header.last_final_block
        // The finalized block is not immediately available so we wait for it to become available.
        let lightClientBlock = null
        let currentValidators = null
        while (lodash.isEmpty(lightClientBlock)) {
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
          if (lodash.isEmpty(lightClientBlock)) {
            await sleep(300)
          }
        }
        console.log('Initializing with validators')
        console.log(`${JSON.stringify(currentValidators)}`)
        const borshInitialValidators = borshifyInitialValidators(
          currentValidators
        )
        let gasPrice = new BN(await this.web3.eth.getGasPrice()).mul(new BN(ethGasMultiplier))
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
                new BN(ethGasMultiplier)
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

  // TODO: Add cli command that allows withdraw funds from client.
  async withdraw ({
    ethGasMultiplier
  }) {
    const web3 = this.web3
    await this.clientContract.methods.withdraw().send({
      from: this.ethMasterAccount,
      gas: 1000000,
      handleRevert: true,
      gasPrice: new BN(await web3.eth.getGasPrice()).mul(new BN(ethGasMultiplier))
    })
  }

  async runInternal ({
    submitInvalidBlock,
    near2ethRelayMinDelay,
    near2ethRelayMaxDelay,
    near2ethRelayErrorDelay,
    near2ethRelaySelectDuration,
    ethGasMultiplier,
    ethUseEip1559
  }) {
    const clientContract = this.clientContract
    const robustWeb3 = this.robustWeb3
    const near = this.near
    const ethMasterAccount = this.ethMasterAccount
    const web3 = this.web3

    const minDelay = Number(near2ethRelayMinDelay)
    const maxDelay = Number(near2ethRelayMaxDelay)
    const errorDelay = Number(near2ethRelayErrorDelay)

    const selectDuration = web3.utils.toBN(Number(near2ethRelaySelectDuration) * 1000_000_000)

    ethGasMultiplier = Number(ethGasMultiplier)
    ethUseEip1559 = ethUseEip1559 === 'true'

    const httpPrometheus = new HttpPrometheus(this.metricsPort, 'near_bridge_near2eth_')
    const clientHeightGauge = httpPrometheus.gauge('client_height', 'amount of block client processed')
    const chainHeightGauge = httpPrometheus.gauge('chain_height', 'current chain height')

    let firstSeenBlockTimestamp = null
    const nextBlockSelection = {
      borshBlock: null,
      height: 0,
      set: function ({ borshBlock, lightClientBlock }) {
        this.borshBlock = borshBlock
        this.height = lightClientBlock.inner_lite.height
        console.log(`The new optimal block is found. Height: ${this.height}. Size: ${this.borshBlock.length} bytes`)
      },
      clean: function () {
        this.borshBlock = null
        this.height = 0
      },
      isEmpty: function () {
        return !this.borshBlock
      },
      isSuitable: function ({ borshBlock, lightClientBlock }) {
        return !this.isEmpty() &&
               this.height !== lightClientBlock.inner_lite.height &&
               this.borshBlock.length >= borshBlock.length
      }
    }
    const getGasOptions = async (useEip1559, gasMultiplier) => {
      const gasOptions = {}
      if (useEip1559) {
        const feeData = await robustWeb3.getFeeData(gasMultiplier)
        gasOptions.maxPriorityFeePerGas = feeData.maxPriorityFeePerGas
        gasOptions.maxFeePerGas = feeData.maxFeePerGas
      } else {
        const gasPrice = new BN(await web3.eth.getGasPrice())
        gasOptions.gasPrice = gasPrice.mul(new BN(gasMultiplier))
      }
      return gasOptions
    }
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
        let replaceDelay = web3.utils.toBN(0)
        if (!nextValidAt.isZero()) {
          replaceDelay = web3.utils
            .toBN(bridgeState.nextTimestamp)
            .add(replaceDuration)
            .sub(web3.utils.toBN(lastBlock.inner_lite.timestamp))
        }
        firstSeenBlockTimestamp = firstSeenBlockTimestamp || lastBlock.inner_lite.timestamp
        // console.log({bridgeState, currentBlockHash, lastBlock, replaceDuration}) // DEBUG
        if (bridgeState.currentHeight < lastBlock.inner_lite.height) {
          if (nextValidAt.isZero() || replaceDelay.cmpn(0) <= 0) {
            // Serialize once here to avoid multiple 'borshify(...)' function calls
            const blockCouple = {
              lightClientBlock: lastBlock,
              borshBlock: borshify(lastBlock)
            }

            // Describe the selection logic here so as not to complicate the reading of the code
            if (nextBlockSelection.isEmpty()) {
              console.log('The selection of the optimal block has begun.')
              nextBlockSelection.set(blockCouple)
            } else if (nextBlockSelection.isSuitable(blockCouple)) {
              nextBlockSelection.set(blockCouple)
            }

            // Calculation of selection delay starting from the first block seen after service restart
            let selectDelay = selectDuration
              .add(web3.utils.toBN(firstSeenBlockTimestamp))
              .sub(web3.utils.toBN(lastBlock.inner_lite.timestamp))
            if (!nextValidAt.isZero()) {
              // Make sure that every time after restarting the service we have a delay for block selection
              selectDelay = web3.utils.BN.max(
                selectDelay,
                selectDuration.add(replaceDelay)
              )
            }
            if (selectDelay.cmpn(0) > 0) {
              const selectDelaySeconds = selectDelay.div(web3.utils.toBN(1000_000_000))
              console.log(`Last light client block: ${lastBlock.inner_lite.height}`)
              console.log(`Time left to make a decision: ${selectDelaySeconds.toString()} seconds`)
              await sleep(1000) // Block creation time is approximately one second, is an additional cli argument needed here?
              continue
            } else {
              console.log('Time to make a decision is over!')
            }

            console.log(
              `Trying to submit new block at height ${nextBlockSelection.height}`
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
              const gasOptions = await getGasOptions(ethUseEip1559, ethGasMultiplier)
              await clientContract.methods.deposit().send({
                from: ethMasterAccount,
                gas: 1000000,
                handleRevert: true,
                value: new BN(lockEthAmount),
                ...gasOptions
              })
              console.log('Transferred.')
            }

            if (submitInvalidBlock) {
              console.log('Mutate block by one byte')
              console.log(nextBlockSelection.borshBlock)
              nextBlockSelection.borshBlock[Math.floor(nextBlockSelection.borshBlock.length * Math.random())] += 1
            }

            const gasOptions = await getGasOptions(ethUseEip1559, ethGasMultiplier)
            await clientContract.methods.addLightClientBlock(nextBlockSelection.borshBlock).send({
              from: ethMasterAccount,
              gas: 10000000,
              handleRevert: true,
              ...gasOptions
            })

            if (submitInvalidBlock) {
              console.log('Successfully submit invalid block')
              return process.exit(0)
            }

            console.log('Submitted.')
            nextBlockSelection.clean()
            await sleep(240000) // To prevent submitting the same block again
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
        clientHeightGauge.set(Number(BigInt(bridgeState.currentHeight)))
        chainHeightGauge.set(Number(BigInt(lastBlock.inner_lite.height)))

        const status = await this.near.connection.provider.sendJsonRpc('status', '')
        console.log(`Last valid header on the client: ${bridgeState.currentHeight}. Next light client block: ${lastBlock.inner_lite.height}`)
        console.log(`Chain height: ${status.sync_info.latest_block_height} Sleeping for ${delay} seconds.`)
        await sleep(1000 * delay)
      } catch (e) {
        console.log('Error', e)
        await sleep(1000 * errorDelay)
      }
    }
  }

  DANGERsubmitInvalidNearBlock (options) {
    return this.runInternal({
      ...options,
      submitInvalidBlock: true
    })
  }

  run (options) {
    return this.runInternal({
      ...options,
      submitInvalidBlock: false
    })
  }
}

exports.Near2EthRelay = Near2EthRelay
exports.borshify = borshify
