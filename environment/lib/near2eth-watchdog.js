const Web3 = require('web3')
const BN = require('bn.js')
const fs = require('fs')
const { RainbowConfig } = require('./config')
const {
  sleep,
  RobustWeb3,
  normalizeEthKey,
  promiseWithTimeout,
} = require('../lib/robust')
const Tx = require('ethereumjs-tx').Transaction

const SLOW_TX_ERROR_MSG = 'transaction not executed within 5 minutes'

class Near2EthWatchdog {
  async initialize() {
    // @ts-ignore
    this.robustWeb3 = new RobustWeb3(RainbowConfig.getParam('eth-node-url'))
    this.web3 = this.robustWeb3.web3
    const ethMasterAccount = this.web3.eth.accounts.privateKeyToAccount(
      normalizeEthKey(RainbowConfig.getParam('eth-master-sk'))
    )
    this.web3.eth.accounts.wallet.add(ethMasterAccount)
    this.web3.eth.defaultAccount = ethMasterAccount.address
    this.ethMasterAccount = ethMasterAccount.address

    // Initialize client contract.
    console.log('Deploying Near2EthClient contract.')
    this.clientContract = new this.web3.eth.Contract(
      // @ts-ignore
      JSON.parse(
        fs.readFileSync(RainbowConfig.getParam('eth-client-abi-path'))
      ),
      RainbowConfig.getParam('eth-client-address'),
      {
        from: this.ethMasterAccount,
        handleRevert: true,
      }
    )
  }

  async run() {
    let privateKey = RainbowConfig.getParam('eth-master-sk')
    if (privateKey.startsWith('0x')) {
      privateKey = privateKey.slice(2)
    }
    privateKey = Buffer.from(privateKey, 'hex')
    while (true) {
      const lastClientBlock = await this.clientContract.methods.last().call()
      const latestBlock = await this.robustWeb3.getBlock('latest')
      console.log(
        `Examining block ${lastClientBlock.hash} height: ${lastClientBlock.height}`
      )
      if (latestBlock.timestamp >= lastClientBlock.valid) {
        const timeDelta = 10
        console.log(`Block is valid. Sleeping for ${timeDelta} seconds.`)
        await sleep(timeDelta * 1000)
        continue
      }

      // We cannot memorize processed blocks because they might have been re-submitted with different data.
      for (let i = 0; i < lastClientBlock.approvals_after_next_length; i++) {
        console.log(`Checking signature ${i}.`)
        const result = await this.clientContract.methods
          .checkBlockProducerSignatureInLastBlock(i)
          .call()
        if (!result) {
          console.log(`Challenging signature ${i}.`)
          try {
            let gasPrice = await this.web3.eth.getGasPrice()
            let nonce = await this.web3.eth.getTransactionCount(
              this.ethMasterAccount
            )
            while (gasPrice < 10000 * 1e9) {
              try {
                // Keep sending with same nonce but higher gasPrice to override same txn
                let tx = new Tx({
                  from: this.ethMasterAccount.address,
                  // this is required otherwise gas is infinite
                  to: RainbowConfig.getParam('eth-client-address'),
                  gasLimit: Web3.utils.toHex(2000000),
                  gasPrice: Web3.utils.toHex(gasPrice),
                  nonce: Web3.utils.toHex(nonce),
                  data: this.clientContract.methods
                    .challenge(this.ethMasterAccount, i)
                    .encodeABI(),
                })
                tx.sign(privateKey)
                tx = '0x' + tx.serialize().toString('hex')

                await promiseWithTimeout(
                  5 * 60 * 1000,
                  this.web3.eth.sendSignedTransaction(tx),
                  SLOW_TX_ERROR_MSG
                )
                break
              } catch (e) {
                if (e.message === SLOW_TX_ERROR_MSG) {
                  console.log(SLOW_TX_ERROR_MSG)
                  console.log(
                    `current gasPrice: ${gasPrice}. rechallenge with double gasPrice`
                  )
                  gasPrice *= 2
                } else {
                  throw e
                }
              }
            }
          } catch (err) {
            console.log(
              `Challenge failed. Maybe the block was already reverted? ${err}`
            )
          }
          break
        }
      }
      const timeDelta = 10
      console.log(`Sleeping for ${timeDelta} seconds`)
      await sleep(timeDelta * 1000)
    }
  }
}

exports.Near2EthWatchdog = Near2EthWatchdog
