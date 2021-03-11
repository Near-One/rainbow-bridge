const BN = require('bn.js')
const fs = require('fs')
const { Web3, normalizeEthKey } = require('rainbow-bridge-utils')

class EthContractInitializer {
  async execute ({
    args,
    gas,
    ethNodeUrl,
    ethMasterSk,
    ethContractAbiPath,
    ethContractBinPath,
    ethGasMultiplier
  }) {
    let ethContractAddress
    if (!ethContractAbiPath || !ethContractBinPath) {
      return null
    }

    try {
      const web3 = new Web3(ethNodeUrl)
      let ethMasterAccount = web3.eth.accounts.privateKeyToAccount(
        normalizeEthKey(ethMasterSk)
      )
      web3.eth.accounts.wallet.add(ethMasterAccount)
      web3.eth.defaultAccount = ethMasterAccount.address
      ethMasterAccount = ethMasterAccount.address

      console.log('Deploying ETH contract')
      const tokenContract = new web3.eth.Contract(
        JSON.parse(fs.readFileSync(ethContractAbiPath))
      )
      const txContract = await tokenContract
        .deploy({
          data: '0x' + fs.readFileSync(ethContractBinPath),
          arguments: args
        })
        .send({
          from: ethMasterAccount,
          gas,
          gasPrice: new BN(await web3.eth.getGasPrice()).mul(new BN(ethGasMultiplier))
        })
      ethContractAddress = normalizeEthKey(txContract.options.address)
      console.log(`Deployed ETH contract to ${ethContractAddress}`)
      try {
        // Only WebSocket provider can close.
        web3.currentProvider.connection.close()
      } catch (e) {}
    } catch (e) {
      console.log(e)
      return null
    }
    return { ethContractAddress }
  }
}

class InitEthEd25519 {
  static async execute ({
    ethNodeUrl,
    ethMasterSk,
    ethEd25519AbiPath,
    ethEd25519BinPath,
    ethGasMultiplier
  }) {
    const ethContractInitializer = new EthContractInitializer()
    const success = await ethContractInitializer.execute(
      {
        args: [],
        gas: 5000000,
        ethContractAbiPath: ethEd25519AbiPath,
        ethContractBinPath: ethEd25519BinPath,
        ethNodeUrl,
        ethMasterSk,
        ethGasMultiplier
      }
    )
    if (!success) {
      console.log("Can't deploy", ethEd25519AbiPath)
      process.exit(1)
    }
    return {
      ethEd25519Address: success.ethContractAddress
    }
  }
}

class InitEthErc20 {
  static async execute ({
    ethNodeUrl,
    ethMasterSk,
    ethErc20AbiPath,
    ethErc20BinPath,
    ethGasMultiplier
  }) {
    const ethContractInitializer = new EthContractInitializer()
    const success = await ethContractInitializer.execute(
      {
        args: [],
        gas: 3000000,
        ethContractAbiPath: ethErc20AbiPath,
        ethContractBinPath: ethErc20BinPath,
        ethNodeUrl,
        ethMasterSk,
        ethGasMultiplier
      }
    )
    if (!success) {
      console.log("Can't deploy", ethErc20AbiPath)
      process.exit(1)
    }
    return {
      ethErc20Address: success.ethContractAddress
    }
  }
}

class InitEthLocker {
  static async execute ({
    ethNodeUrl,
    ethMasterSk,
    nearTokenFactoryAccount,
    ethProverAddress,
    ethLockerAbiPath,
    ethLockerBinPath,
    ethAdminAddress,
    ethGasMultiplier
  }) {
    if (ethAdminAddress === '') {
      const web3 = new Web3('')
      ethAdminAddress = web3.eth.accounts.privateKeyToAccount(ethMasterSk).address
    }

    console.log('Using as locker admin:', ethAdminAddress)
    const ethContractInitializer = new EthContractInitializer()
    const minBlockAcceptanceHeight = 0
    const pausedFlag = 0

    const success = await ethContractInitializer.execute(
      {
        args: [
          Buffer.from(nearTokenFactoryAccount, 'utf8'),
          ethProverAddress,
          minBlockAcceptanceHeight,
          ethAdminAddress,
          pausedFlag
        ],
        gas: 5000000,
        ethContractAbiPath: ethLockerAbiPath,
        ethContractBinPath: ethLockerBinPath,
        ethNodeUrl,
        ethMasterSk,
        ethGasMultiplier
      }
    )
    if (!success) {
      console.log("Can't deploy", ethLockerAbiPath)
      process.exit(1)
    }
    return {
      ethLockerAddress: success.ethContractAddress,
      ethAdminAddress: ethAdminAddress
    }
  }
}

class InitEthClient {
  static async execute ({
    ethNodeUrl,
    ethMasterSk,
    ethClientLockEthAmount,
    ethClientLockDuration,
    ethClientReplaceDuration,
    ethEd25519Address,
    ethClientAbiPath,
    ethClientBinPath,
    ethAdminAddress,
    ethGasMultiplier
  }) {
    if (ethAdminAddress === '') {
      const web3 = new Web3('')
      ethAdminAddress = web3.eth.accounts.privateKeyToAccount(ethMasterSk)
        .address
    }

    ethClientLockDuration = Number(ethClientLockDuration)
    ethClientReplaceDuration = Number(ethClientReplaceDuration)

    // replace duration should be at least twice as long as lock duration or 20 minutes longer
    const minAllowedReplaceDuration = Math.min(
      ethClientLockDuration + 20 * 60,
      2 * ethClientLockDuration
    )

    if (ethClientReplaceDuration < minAllowedReplaceDuration) {
      throw new Error(
        `Invalid parameters ${JSON.stringify({
          ethClientLockDuration,
          ethClientReplaceDuration,
          minAllowedReplaceDuration
        })}`
      )
    }

    const ethContractInitializer = new EthContractInitializer()
    const web3 = new Web3(ethNodeUrl)
    const lockEthAmount = web3.utils.toBN(ethClientLockEthAmount)
    const lockDuration = web3.utils.toBN(ethClientLockDuration)
    const replaceDuration = web3.utils
      .toBN(ethClientReplaceDuration)
      .mul(new web3.utils.BN(1e9))
    try {
      // Only WebSocket provider can close.
      web3.currentProvider.connection.close()
    } catch (e) {}
    const success = await ethContractInitializer.execute(
      {
        args: [
          ethEd25519Address,
          lockEthAmount,
          lockDuration,
          replaceDuration,
          ethAdminAddress,
          0
        ],
        gas: 5000000,
        ethContractAbiPath: ethClientAbiPath,
        ethContractBinPath: ethClientBinPath,
        ethNodeUrl,
        ethMasterSk,
        ethGasMultiplier
      }
    )
    if (!success) {
      console.log("Can't deploy", ethClientAbiPath)
      process.exit(1)
    }
    return {
      ethClientAddress: success.ethContractAddress
    }
  }
}

class InitEthProver {
  static async execute ({
    ethNodeUrl,
    ethMasterSk,
    ethClientAddress,
    ethProverAbiPath,
    ethProverBinPath,
    ethAdminAddress,
    ethGasMultiplier
  }) {
    if (ethAdminAddress === '') {
      const web3 = new Web3('')
      ethAdminAddress = web3.eth.accounts.privateKeyToAccount(ethMasterSk)
        .address
    }

    const ethContractInitializer = new EthContractInitializer()
    const success = await ethContractInitializer.execute(
      {
        args: [ethClientAddress, ethAdminAddress, 0],
        gas: 3000000,
        ethContractAbiPath: ethProverAbiPath,
        ethContractBinPath: ethProverBinPath,
        ethNodeUrl,
        ethMasterSk,
        ethGasMultiplier
      }
    )
    if (!success) {
      console.log("Can't deploy", ethProverAbiPath)
      process.exit(1)
    }
    return {
      ethProverAddress: success.ethContractAddress
    }
  }
}

exports.InitEthEd25519 = InitEthEd25519
exports.InitEthErc20 = InitEthErc20
exports.InitEthLocker = InitEthLocker
exports.InitEthClient = InitEthClient
exports.InitEthProver = InitEthProver
