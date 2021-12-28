const BN = require('bn.js')
const fs = require('fs')
const { Web3, normalizeEthKey, sleep, execAsync } = require('rainbow-bridge-utils')

const RETRY_SEND_TX = 15

class EthContractInitializer {
  async execute ({
    args,
    gas,
    ethNodeUrl,
    ethMasterSk,
    ethContractAbiPath,
    ethContractBinPath,
    ethContractArtifactPath,
    ethGasMultiplier
  }) {
    let ethContractAddress
    if ((!ethContractAbiPath || !ethContractBinPath) && !ethContractArtifactPath) {
      return null
    }

    for (let i = 0; i < RETRY_SEND_TX; i++) {
      try {
        const web3 = new Web3(ethNodeUrl)
        let ethMasterAccount = web3.eth.accounts.privateKeyToAccount(
          normalizeEthKey(ethMasterSk)
        )
        web3.eth.accounts.wallet.add(ethMasterAccount)
        web3.eth.defaultAccount = ethMasterAccount.address
        ethMasterAccount = ethMasterAccount.address

        console.log('Deploying ETH contract')
        let abi, bytecode
        if (ethContractArtifactPath) {
          ({ abi, bytecode } = JSON.parse(fs.readFileSync(ethContractArtifactPath)))
        } else {
          abi = JSON.parse(fs.readFileSync(ethContractAbiPath))
          bytecode = '0x' + fs.readFileSync(ethContractBinPath)
        }
        const tokenContract = new web3.eth.Contract(abi)
        const txContract = await tokenContract
          .deploy({
            data: bytecode,
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
        } catch (e) {
        }
        return { ethContractAddress }
      } catch (e) {
        if (e.message.indexOf('the tx doesn\'t have the correct nonce') >= 0 ||
            e.message.indexOf('replacement transaction underpriced') >= 0) {
          console.log('nonce error, retrying...')
          await sleep(5 * 1000)
          continue
        }

        console.log(e)
        return null
      }
    }
  }
}

class InitEthEd25519 {
  static async execute ({
    ethNodeUrl,
    ethMasterSk,
    ethEd25519ArtifactPath,
    ethGasMultiplier
  }) {
    const ethContractInitializer = new EthContractInitializer()
    const success = await ethContractInitializer.execute(
      {
        args: [],
        gas: 5000000,
        ethContractArtifactPath: ethEd25519ArtifactPath,
        ethNodeUrl,
        ethMasterSk,
        ethGasMultiplier
      }
    )
    if (!success) {
      console.log("Can't deploy", ethEd25519ArtifactPath)
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
    ethClientLockEthAmount,
    ethClientLockDuration,
    ethClientReplaceDuration,
    ethEd25519Address
  }) {
    console.log('Start deploy ETH client proxy')

    const cmd = `
    cd ./contracts/eth/nearbridge && \\
    npx hardhat deployNearBridgeProxy \\
    --ed25519 ${ethEd25519Address} \\
    --eth-client-lock-eth-amount ${ethClientLockEthAmount} \\
    --eth-client-lock-duration ${ethClientLockDuration} \\
    --eth-client-replace-duration ${ethClientReplaceDuration} \\
    --paused-flags 0 \\
    --config rainbowBridgeConfig.js \\
    --network rainbowBridge
    `
    await execAsync(cmd)

    console.log('ETH client proxy deployed!')
  }
}

class VerifyAddress {
  static async execute (address) {
    console.log(`Start verify contract address ${address}`)

    const cmd = `cd ./contracts/eth/nearbridge && npx hardhat verify ${address} \\
    --config rainbowBridgeConfig.js --network rainbowBridge
    `
    await execAsync(cmd)

    console.log(`Contract address ${address} verified!`)
  }
}

class InitEthProver {
  static async execute ({
    ethClientAddress
  }) {
    console.log('Start deploy ETH prover proxy')

    const cmd = `
    cd ./contracts/eth/nearprover && npx hardhat deployNearProverProxy \\
    --eth-client-address ${ethClientAddress} \\
    --paused-flags 0 \\
    --config rainbowBridgeConfig.js \\
    --network rainbowBridge
    `
    await execAsync(cmd)

    console.log('ETH prover proxy deployed!')
  }
}

exports.InitEthEd25519 = InitEthEd25519
exports.InitEthErc20 = InitEthErc20
exports.InitEthLocker = InitEthLocker
exports.InitEthClient = InitEthClient
exports.InitEthProver = InitEthProver
exports.VerifyAddress = VerifyAddress
