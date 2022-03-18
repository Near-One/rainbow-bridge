require('@nomiclabs/hardhat-ethers');
require('solidity-coverage');

require('dotenv').config();

const ETH_PRIVATE_KEY = process.env.ETH_PRIVATE_KEY || '11'.repeat(32);
const INFURA_API_KEY = process.env.INFURA_API_KEY || '';

task('upgrade-provers-bridge-address-to', 'Upgrades the provided prover to use the bridge at the provided address')
    .addParam('prover', 'Prover address')
    .addParam('newBridge', 'The address of the new bridge')
    .addParam('ledgerKeyPath', 'The ledger key path to sign transactions', undefined, undefined, true)
    .setAction(async (taskArgs, hre) => {
        const { upgradeProversBridgeAddressTo } = require('./utils/upgrade_bridge_address.js');
        await upgradeProversBridgeAddressTo(hre.ethers.provider, taskArgs.prover, taskArgs.newBridge, taskArgs.ledgerKeyPath);
    });

task('get-provers-bridge-address', 'Returns the current bridge address used in the prover at the provided address')
    .addParam('prover', 'Prover address')
    .setAction(async taskArgs => {
        const { getProversBridgeAddress } = require('./utils/upgrade_bridge_address.js');
        const bridgeAddress = await getProversBridgeAddress(taskArgs.prover);
        console.log(`Prover address: ${taskArgs.prover}`);
        console.log(`Current bridge address: ${bridgeAddress}`);
    });

task('upgrade-admin-to', 'Upgrades the admin address on X contract to the provided address')
    .addParam('contractAddress', 'Contract address')
    .addParam('currentAdminAddress', 'The address of the current admin')
    .addParam('newAdminAddress', 'The address of the new admin')
    .addParam('slot', 'The admin address slot', undefined, undefined, types.int)
    .addParam('ledgerKeyPath', 'The ledger key path to sign transactions', undefined, types.string, true)
    .setAction(async (taskArgs, hre) => {
        const { upgradeAdminAddressTo } = require('./utils/upgrade_admin.js');
        await upgradeAdminAddressTo({
            provider: hre.ethers.provider,
            contractAddress: taskArgs.contractAddress,
            currentAdminAddress: taskArgs.currentAdminAddress,
            newAdminAddress: taskArgs.newAdminAddress,
            adminAddressSlot: taskArgs.slot,
            ledgerKeyPath: taskArgs.ledgerKeyPath,
        });
    });

task('get-slots-data', 'Display slots')
    .addParam('contractAddress', 'Contract address')
    .addParam('numOfSlotsToDisplay', 'Number of slots to fetch')
    .setAction(async (taskArgs, hre) => {
        const { getSlotsData } = require('./utils/upgrade_admin.js');
        await getSlotsData(hre.ethers.provider, taskArgs.contractAddress, taskArgs.numOfSlotsToDisplay);
    });

/**
 * @type import('hardhat/config').HardhatUserConfig
 */
module.exports = {
    solidity: {
        version: '0.8.11',
        settings: {
            optimizer: {
                enabled: true,
                runs: 1000
            }
        }
    },
    networks: {
        ropsten: {
            url: `https://ropsten.infura.io/v3/${INFURA_API_KEY}`,
            accounts: [`0x${ETH_PRIVATE_KEY}`],
            gasPrice: 50000000000,
            gasMultiplier: 2,
        },
        mainnet: {
            url: `https://mainnet.infura.io/v3/${INFURA_API_KEY}`,
            accounts: [`0x${ETH_PRIVATE_KEY}`],
        },
        goerli: {
            url: `https://goerli.infura.io/v3/${INFURA_API_KEY}`,
            accounts: [`0x${ETH_PRIVATE_KEY}`],
            gasPrice: 50000000000,
            gasMultiplier: 2,
        },
    }
};
