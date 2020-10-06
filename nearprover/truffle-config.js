module.exports = {
    // See <http://truffleframework.com/docs/advanced/configuration>
    // to customize your Truffle configuration!
    networks: {
        development: {
            host: 'localhost',
            port: 9545,
            network_id: '*',
            gas: 8000000,
            gasPrice: 1000000000, // web3.eth.gasPrice
        },
        coverage: {
            host: 'localhost',
            port: 8555,
            network_id: '*',
            gas: 8000000,
            gasPrice: 1000000000, // web3.eth.gasPrice
        },
        soliditycoverage: {
            port: 8555,
            host: "localhost",
            network_id: "*",
        }
    },
    compilers: {
        solc: {
            version: '0.6.12',
            settings: {
                optimizer: {
                    enabled: true,
                    runs: 200,
                }
            }
        },
    },
    mocha: { // https://github.com/cgewecke/eth-gas-reporter
        reporter: 'eth-gas-reporter',
        reporterOptions : {
            codechecks: true,
            currency: 'USD',
            gasPrice: 10,
            onlyCalledMethods: true,
            showTimeSpent: true,
            excludeContracts: ['Migrations']
        }
    },
    plugins: ["solidity-coverage"]
};
