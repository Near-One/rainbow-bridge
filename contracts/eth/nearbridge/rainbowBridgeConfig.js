require('@nomiclabs/hardhat-ethers')
require('@nomiclabs/hardhat-etherscan')
require('@openzeppelin/hardhat-upgrades')
require('@openzeppelin/hardhat-upgrades')
require('solidity-coverage')

const path = require('path')
const fs = require('fs')
const os = require('os')
const { task } = require('hardhat/config')
const {
  deployNearBridgeProxy
} = require('./scripts/tasks')

const ETHERSCAN_API_KEY = process.env.ETHERSCAN_API_KEY;

task('deployNearBridgeProxy', 'Deploy NearBridge proxy')
  .addParam('ethClientLockEthAmount', 'ethClientLockEthAmount')
  .addParam('ethClientLockDuration', 'ethClientLockDuration')
  .addParam('ethClientReplaceDuration', 'ethClientReplaceDuration')
  .addParam('ed25519', 'ed25519 address')
  .addParam('pausedFlags', 'pausedFlags')
  .addOptionalParam('admin', 'admin')
  .setAction(async (args, hre) => {
    await deployNearBridgeProxy(hre, 
      args
    )
  })

function setupRainbowBridgeNetwork () {
  const p = path.join(os.homedir(), '.rainbow/config.json')
  const cfg = fs.readFileSync(p)
  const rainbowConfig = JSON.parse(cfg)
  return {
    url: rainbowConfig.ethNodeUrl,
    accounts: [rainbowConfig.ethMasterSk],
    gasPrice: 10000000000,
    gas: 10000000
  }
}

module.exports = {
  defaultNetwork: 'rainbowBridge',
  solidity: {
    version: '0.8.7',
    settings: {
      optimizer: {
        enabled: true,
        runs: 1000
      }
    }
  },
  networks: {
    rainbowBridge: setupRainbowBridgeNetwork()
  },
  etherscan: {
    apiKey: ETHERSCAN_API_KEY 
  }
}
