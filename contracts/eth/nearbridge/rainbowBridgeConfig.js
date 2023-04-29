require('@nomiclabs/hardhat-ethers');
require('@nomiclabs/hardhat-etherscan');
require('@openzeppelin/hardhat-upgrades');
require('solidity-coverage');
require('@openzeppelin/hardhat-defender');
require('dotenv').config();
require('@nomiclabs/hardhat-etherscan');

const path = require('path');
const fs = require('fs');
const os = require('os');
const { task } = require('hardhat/config');


task('deployNearBridgeProxy', 'Deploy NearBridge proxy')
    .addParam('ethclientlockethamount', 'ethClientLockEthAmount')
    .addParam('ethclientlockduration', 'ethClientLockDuration')
    .addParam('ethclientreplaceduration', 'ethClientReplaceDuration')
    .addParam('ed25519', 'ed25519 address')
    .addParam('pausedflags', 'pausedFlags')
    .addParam('upgrader', 'Upgrader Contract Address')
    .addOptionalParam('admin', 'admin')
    .setAction(async (args, hre) => {
        const { deployNearBridgeProxy } = require('./scripts/tasks');
        await deployNearBridgeProxy(hre,
            args,
        );
    });

task('transferOwnership', 'Transfer the ownership of near-bridge contract')
    .addParam('currentadmin', 'Current owner address of near-bridge contract')
    .addParam('newadmin', 'New owner address to set for near-bridge contract')
    .addParam('bridgeaddress', 'Near bridge contract address')
    .setAction(async (args, hre) => {
        const { transferOwnership } = require('./scripts/tasks');
        await hre.run('compile');
        await transferOwnership(args.currentadmin, args.newadmin, args.bridgeaddress);
    });

task('proposeUpgrade', 'Propose new implementation upgrade for near-bridge contract')
    .addParam('proxyaddress', 'Proxy address of near-bridge contract')
    .addParam('newcontractname', 'New implementation of near-bridge contract name')
    .addParam('upgrader', 'upgrader admin address')
    .setAction(async (args, hre) => {
        const { proposeUpgrade } = require('./scripts/tasks');
        await hre.run('compile');
        await proposeUpgrade(args.proxyaddress, args.newcontractname, args.upgrader);
    });

function setupRainbowBridgeNetwork () {
    const p = path.join(os.homedir(), '.rainbow/config.json');
    const cfg = fs.readFileSync(p);
    const rainbowConfig = JSON.parse(cfg);
    return {
        url: rainbowConfig.ethNodeUrl,
        accounts: [rainbowConfig.ethMasterSk],
        gas: 10000000,
    };
}

module.exports = {
    defaultNetwork: 'rainbowBridge',
    solidity: {
        version: '0.8.7',
        settings: {
            optimizer: {
                enabled: true,
                runs: 1000,
            },
        },
    },
    networks: {
        rainbowBridge: setupRainbowBridgeNetwork(),
        localnet: {
            url: 'HTTP://127.0.0.1:8545',
            allowUnlimitedContractSize: true,
        }
    },
    etherscan: {
        apiKey: process.env.ETHERSCAN_API_KEY, // ETHERSCAN_API_KEY
    },
    defender: {
        apiKey: process.env.DEFENDER_TEAM_API_KEY,
        apiSecret: process.env.DEFENDER_TEAM_API_SECRET_KEY,
    }
};
