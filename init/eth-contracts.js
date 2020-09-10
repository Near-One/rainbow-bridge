const Web3 = require('web3')
const BN = require('bn.js')
const fs = require('fs')
const { RainbowConfig } = require('../config')
const { normalizeEthKey } = require('../rainbow/robust')

class EthContractInitializer {
  async execute(contractName, args, gas) {
    const address = 'eth-' + contractName + '-address'
    const abiPath = RainbowConfig.getParam('eth-' + contractName + '-abi-path')
    const binPath = RainbowConfig.getParam('eth-' + contractName + '-bin-path')
    if (!abiPath || !binPath) {
      return false
    }

    try {
      const web3 = new Web3(RainbowConfig.getParam('eth-node-url'))
      let ethMasterAccount = web3.eth.accounts.privateKeyToAccount(
        normalizeEthKey(RainbowConfig.getParam('eth-master-sk'))
      )
      web3.eth.accounts.wallet.add(ethMasterAccount)
      web3.eth.defaultAccount = ethMasterAccount.address
      ethMasterAccount = ethMasterAccount.address

      console.log('Deploying ETH contract', contractName)
      const tokenContract = new web3.eth.Contract(
        JSON.parse(fs.readFileSync(abiPath))
      )
      const txContract = await tokenContract
        .deploy({
          data: '0x' + fs.readFileSync(binPath),
          arguments: args,
        })
        .send({
          from: ethMasterAccount,
          gas: gas,
          gasPrice: new BN(await web3.eth.getGasPrice()).mul(
            new BN(RainbowConfig.getParam('eth-gas-multiplier'))
          ),
        })
      console.log(
        'Deployed ETH contract',
        contractName,
        'to',
        `${txContract.options.address}`
      )
      RainbowConfig.setParam(address, txContract.options.address)
      RainbowConfig.saveConfig()
      try {
        // Only WebSocket provider can close.
        web3.currentProvider.connection.close()
      } catch (e) {}
    } catch (e) {
      console.log(e)
      return false
    }
    return true
  }
}

class InitEthEd25519 {
  static async execute() {
    const ethContractInitializer = new EthContractInitializer()
    const contractName = 'ed25519'
    const success = await ethContractInitializer.execute(
      contractName,
      [],
      5000000
    )
    if (!success) {
      console.log("Can't deploy", contractName)
      throw 1
    }
  }
}

class InitEthErc20 {
  static async execute() {
    const ethContractInitializer = new EthContractInitializer()
    const contractName = 'erc20'
    const success = await ethContractInitializer.execute(
      contractName,
      [],
      3000000
    )
    if (!success) {
      console.log("Can't deploy", contractName)
      throw 1
    }
  }
}

class InitEthLocker {
  static async execute() {
    const ethContractInitializer = new EthContractInitializer()
    const contractName = 'locker'
    const success = await ethContractInitializer.execute(
      contractName,
      [
        Buffer.from(RainbowConfig.getParam('near-fun-token-account'), 'utf8'),
        RainbowConfig.getParam('eth-prover-address'),
      ],
      3000000
    )
    if (!success) {
      console.log("Can't deploy", contractName)
      throw 1
    }
  }
}

class InitEthClient {
  static async execute() {
    const ethContractInitializer = new EthContractInitializer()
    const contractName = 'client'
    const web3 = new Web3(RainbowConfig.getParam('eth-node-url'))
    const lockEthAmount = await web3.utils.toBN(
      RainbowConfig.getParam('eth-client-lock-eth-amount')
    )
    const lockDuration = await web3.utils.toBN(
      RainbowConfig.getParam('eth-client-lock-duration')
    )
    try {
      // Only WebSocket provider can close.
      web3.currentProvider.connection.close()
    } catch (e) {}
    const success = await ethContractInitializer.execute(
      contractName,
      [
        RainbowConfig.getParam('eth-ed25519-address'),
        lockEthAmount,
        lockDuration,
      ],
      3000000
    )
    if (!success) {
      console.log("Can't deploy", contractName)
      throw 1
    }
  }
}

class InitEthProver {
  static async execute() {
    const ethContractInitializer = new EthContractInitializer()
    const contractName = 'prover'
    const success = await ethContractInitializer.execute(
      contractName,
      [RainbowConfig.getParam('eth-client-address')],
      3000000
    )
    if (!success) {
      console.log("Can't deploy", contractName)
      throw 1
    }
  }
}

exports.InitEthEd25519 = InitEthEd25519
exports.InitEthErc20 = InitEthErc20
exports.InitEthLocker = InitEthLocker
exports.InitEthClient = InitEthClient
exports.InitEthProver = InitEthProver
