const nearlib = require('nearlib');
const { maybeCreateAccount, verifyAccount } = require('../lib/near-helpers');
const {
    Eth2NearClientContract,
} = require('../lib/eth2near-client-contract');
const {
    EthProverContract,
} = require('../lib/eth-prover-contract');
const { RainbowConfig } = require('../lib/config');

class InitNEARContracts {
    static async execute () {
        const masterAccount = RainbowConfig.param('near-master-account');
        const masterSk = RainbowConfig.param('near-master-sk');
        const clientAccount = RainbowConfig.param('eth2near-client-account');
        let clientSk = RainbowConfig.param('eth2near-client-sk');
        if (!clientSk) {
            clientSk = masterSk;
        }
        const clientContractPath = RainbowConfig.param('eth2near-client-contract-path');
        const clientInitBalance = RainbowConfig.param('eth2near-client-init-balance');

        const proverAccount = RainbowConfig.param('eth2near-prover-account');
        let proverSk = RainbowConfig.param('eth2near-prover-sk');
        if (!proverSk) {
            proverSk = masterSk;
        }
        const proverContractPath = RainbowConfig.param('eth2near-prover-contract-path');
        const proverInitBalance = RainbowConfig.param('eth2near-prover-init-balance');

        const nearNodeUrl = RainbowConfig.param('near-node-url');
        const nearNetworkId = RainbowConfig.param('near-network-id');
        const validateEthash = RainbowConfig.param('eth2near-client-validate-ethash');

        const clientPk = nearlib.KeyPair.fromString(clientSk).publicKey;
        const proverPk = nearlib.KeyPair.fromString(proverSk).publicKey;

        let keyStore = new nearlib.keyStores.InMemoryKeyStore();
        await keyStore.setKey(nearNetworkId, masterAccount, nearlib.KeyPair.fromString(masterSk));
        await keyStore.setKey(nearNetworkId, clientAccount, nearlib.KeyPair.fromString(clientSk));
        await keyStore.setKey(nearNetworkId, proverAccount, nearlib.KeyPair.fromString(proverSk));
        let near = await nearlib.connect({
            nodeUrl: nearNodeUrl,
            networkId: nearNetworkId,
            masterAccount: masterAccount,
            deps: {
                keyStore: keyStore,
            },
        });

        console.log('Creating accounts and deploying the contracts.');
        await verifyAccount(near, masterAccount);
        await maybeCreateAccount(near, masterAccount, clientAccount, clientPk, clientInitBalance, clientContractPath);
        await verifyAccount(near, clientAccount);
        await maybeCreateAccount(near, masterAccount, proverAccount, proverPk, proverInitBalance, proverContractPath);
        await verifyAccount(near, proverAccount);

        console.log('Initializing client and prover contracts.');
        let clientContract = new Eth2NearClientContract(new nearlib.Account(near.connection, clientAccount), clientAccount);
        await clientContract.maybeInitialize(validateEthash === 'true');

        let proverContract = new EthProverContract(new nearlib.Account(near.connection, proverAccount), proverAccount);
        await proverContract.maybeInitialize(clientAccount);

        RainbowConfig.saveConfig();
    }
}

exports.InitNEARContracts = InitNEARContracts;
