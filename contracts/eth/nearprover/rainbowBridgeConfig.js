require('@nomiclabs/hardhat-ethers')
require('@nomiclabs/hardhat-etherscan')
require('@openzeppelin/hardhat-upgrades')
require('@openzeppelin/hardhat-upgrades')
require('solidity-coverage')

const path = require('path')
const fs = require('fs')
const os = require('os')
const { task } = require('hardhat/config')
const { deployNearProverProxy } = require('./scripts/tasks')

task('deployNearProverProxy', 'Deploy NearProver proxy')
  .addParam('ethClientAddress', 'eth client address')
  // .addParam('admin', 'ethAdminAddress')
  .addParam('pausedFlags', 'pausedFlags')
  .setAction(async (args, hre) => {
    await deployNearProverProxy(hre, args)
  })

function setupRainbowBridgeNetwork () {
  const p = path.join(os.homedir(), '.rainbow/config.json')
  const cfg = fs.readFileSync(p)
  const rainbowConfig = JSON.parse(cfg)
  console.log
  return {
    url: rainbowConfig.ethNodeUrl,
    accounts: [rainbowConfig.ethMasterSk]
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
    apiKey: ''
  }
}
