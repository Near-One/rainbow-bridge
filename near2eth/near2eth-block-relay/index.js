const fs = require('fs')
// @ts-ignore
const bs58 = require('bs58')
// @ts-ignore
const { toBuffer } = require('eth-util-lite')
const { BN } = require('ethereumjs-util')
const {
  sleep,
  RobustWeb3,
  normalizeEthKey,
  borshify,
  borshifyInitialValidators,
  nearAPI
} = require('rainbow-bridge-utils')

class Near2EthRelay {
  async initialize ({
    nearNodeUrl,
    nearNetworkId,
    ethNodeUrl,
    ethMasterSk,
    ethClientAbiPath,
    ethClientAddress,
    ethGasMultiplier
  }) {
    // @ts-ignore
    this.robustWeb3 = new RobustWeb3(ethNodeUrl)
    this.web3 = this.robustWeb3.web3
    this.ethMasterAccount = this.web3.eth.accounts.privateKeyToAccount(
      normalizeEthKey(ethMasterSk)
    )
    this.web3.eth.accounts.wallet.add(this.ethMasterAccount)
    this.web3.eth.defaultAccount = this.ethMasterAccount.address
    this.ethMasterAccount = this.ethMasterAccount.address

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
      // @ts-ignore
      JSON.parse(
        fs.readFileSync(ethClientAbiPath)
      ),
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

  async runInternal ({
    submitInvalidBlock,
    near2ethRelayMinDelay,
    near2ethRelayMaxDelay,
    near2ethRelayErrorDelay,
    ethGasMultiplier
  }) {
    const clientContract = this.clientContract
    const robustWeb3 = this.robustWeb3
    const near = this.near
    const ethMasterAccount = this.ethMasterAccount
    const web3 = this.web3

    const minDelay = Number(near2ethRelayMinDelay)
    const maxDelay = Number(near2ethRelayMaxDelay)
    const errorDelay = Number(near2ethRelayErrorDelay)

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
                gasPrice: new BN(await web3.eth.getGasPrice()).mul(new BN(ethGasMultiplier))
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
              gasPrice: new BN(await web3.eth.getGasPrice()).mul(new BN(ethGasMultiplier))
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
