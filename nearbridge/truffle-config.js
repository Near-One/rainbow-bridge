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
        }
    },
    compilers: {
        solc: {
            version: '0.5.5',
            settings: {
                optimizer: {
                    enabled: true,
                    runs: 200,
                }
            }
        },
    }
};
