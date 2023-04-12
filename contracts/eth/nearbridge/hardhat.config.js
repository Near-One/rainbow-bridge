require('@nomiclabs/hardhat-ethers');
require('@nomiclabs/hardhat-waffle');
require('@openzeppelin/hardhat-upgrades');
require("hardhat-gas-reporter");
require('solidity-coverage');

module.exports = {
    solidity: {
        version: '0.8.7',
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
