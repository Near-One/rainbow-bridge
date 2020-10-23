
function getConfig(env) {
    switch (env) {
        case 'mainnet':
            return {
                networkId: 'mainnet',
                nearNodeUrl: 'https://rpc.mainnet.near.org',
                ethNodeUrl: '',
            };
        case 'testnet':
            return {
                networkId: 'default',
                nearNodeUrl: 'https://rpc.testnet.near.org',
                ethNodeUrl: '',
            };
        case 'local':
        case 'test':
            return {
                networkId: 'local',
                // NEAR configuration.
                nearNodeUrl: 'http://localhost:3030',
                keyPath: `${process.env.HOME}/.near/local/validator_key.json`,
                masterAccount: 'test.near',
                nearEthClientId: 'client.test.near',
                nearEthProverId: 'prover.test.near',
                // Ethereum configuration.
                ethNodeUrl: 'http://localhost:9545',
                ethFromSecretKey: '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200',
                ethProverAddress: '',
            };
        default:
            throw new Error(`Unconfigured environment '${env}'. Can be configured in src/config.js.`);
    }
}

module.exports = getConfig;
