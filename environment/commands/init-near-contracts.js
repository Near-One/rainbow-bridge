const nearlib = require('near-api-js');
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
        const masterAccount = RainbowConfig.getParam('near-master-account');
        const masterSk = RainbowConfig.getParam('near-master-sk');
        const clientAccount = RainbowConfig.getParam('eth2near-client-account');
        let clientSk = RainbowConfig.maybeGetParam('eth2near-client-sk');
        if (!clientSk) {
            console.log('Key to call Eth2NearClient contract is not specified. Reusing master key.');
            clientSk = masterSk;
            RainbowConfig.setParam('eth2near-client-sk', masterSk);
        }
        const clientContractPath = RainbowConfig.getParam('eth2near-client-contract-path');
        const clientInitBalance = RainbowConfig.getParam('eth2near-client-init-balance');

        const proverAccount = RainbowConfig.getParam('eth2near-prover-account');
        let proverSk = RainbowConfig.maybeGetParam('eth2near-prover-sk');
        if (!proverSk) {
            console.log('Key to call Eth2NearProver contract is not specified. Reusing master key.');
            proverSk = masterSk;
            RainbowConfig.setParam('eth2near-prover-sk', masterSk);
        }
        const proverContractPath = RainbowConfig.getParam('eth2near-prover-contract-path');
        const proverInitBalance = RainbowConfig.getParam('eth2near-prover-init-balance');

        const nearNodeUrl = RainbowConfig.getParam('near-node-url');
        const nearNetworkId = RainbowConfig.getParam('near-network-id');
        const validateEthash = RainbowConfig.getParam('eth2near-client-validate-ethash');

        const clientPk = nearlib.KeyPair.fromString(clientSk).getPublicKey();
        const proverPk = nearlib.KeyPair.fromString(proverSk).getPublicKey();

        const keyStore = new nearlib.keyStores.InMemoryKeyStore();
        await keyStore.setKey(nearNetworkId, masterAccount, nearlib.KeyPair.fromString(masterSk));
        await keyStore.setKey(nearNetworkId, clientAccount, nearlib.KeyPair.fromString(clientSk));
        await keyStore.setKey(nearNetworkId, proverAccount, nearlib.KeyPair.fromString(proverSk));
        const near = await nearlib.connect({
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
        const clientContract = new Eth2NearClientContract(new nearlib.Account(near.connection, clientAccount), clientAccount);
        await clientContract.maybeInitialize(validateEthash === 'true');

        const proverContract = new EthProverContract(new nearlib.Account(near.connection, proverAccount), proverAccount);
        await proverContract.maybeInitialize(clientAccount);

        RainbowConfig.saveConfig();
    }
}

exports.InitNEARContracts = InitNEARContracts;
