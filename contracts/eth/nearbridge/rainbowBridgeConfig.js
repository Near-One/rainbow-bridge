require('@nomiclabs/hardhat-ethers');
require("@nomiclabs/hardhat-etherscan");
require('@openzeppelin/hardhat-upgrades');
require('@openzeppelin/hardhat-upgrades');
require('solidity-coverage');

const path = require('path');
const fs = require('fs');
const os = require('os')
const { task } = require('hardhat/config');
const { deployNearBridgeProxy } = require('./scripts/tasks');

task('deployNearBridgeProxy', 'Deploy NearBridge proxy')
    .addParam('ethClientArtifactPath', 'client artifact path.')
    .addParam('ed25519', 'ed25519 address')
    .addParam('privateKey', 'Deployer private key')
    .addParam('lockEthAmount', 'lockEthAmount')
    .addParam('lockDuration', 'lockDuration')
    .addParam('replaceDuration', 'replaceDuration')
    .addParam('admin', 'admin address')
    .addParam('pausedFlags', 'pausedFlags')
    .setAction(async (args, hre) => {
    const data = JSON.parse(await fs.promises.readFile(args.ethClientArtifactPath));
    await deployNearBridgeProxy(hre, {
        abi: data.abi,
        bytecode: data.bytecode,
        ...args,
    });
});

function setupRainbowBridgeNetwork() {
    const p =path.join(os.homedir(), ".rainbow/config.json")
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
        apiKey: ""
    }
};
