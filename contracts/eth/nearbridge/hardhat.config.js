require('@nomiclabs/hardhat-ethers');
require('@nomiclabs/hardhat-waffle');
require("hardhat-gas-reporter");
require('solidity-coverage');

/**
 * @type import('hardhat/config').HardhatUserConfig
 */
module.exports = {
    solidity: {
        version: '0.8.11',
        gasReporter: {
            currency: 'USD',
            enabled: true,
        },
        settings: {
            optimizer: {
                enabled: true,
                runs: 1000
            }
        }
    }
};
