const nearlib = require('nearlib');
const { maybeCreateAccount, verifyAccount } = require('../lib/near-helpers');
const {
    Eth2NearClientContract,
} = require('../lib/eth2near-client-contract');
const {
    EthProverContract,
} = require('../lib/eth-prover-contract');

class InitNEARContracts {
    static async execute (command) {
        const masterAccount = command.masterAccount;
        const masterSk = command.masterSk;
        const clientAccount = command.clientAccount;
        let clientSk = command.clientSk;
        if (!clientSk) {
            clientSk = masterSk;
        }
        const clientContractPath = command.clientContractPath;
        const clientInitBalance = command.clientInitBalance;

        const proverAccount = command.proverAccount;
        let proverSk = command.proverSk;
        if (!proverSk) {
            proverSk = masterSk;
        }
        const proverContractPath = command.proverContractPath;
        const proverInitBalance = command.proverInitBalance;

        const nearNodeUrl = command.nearNodeUrl;
        const nearNetworkId = command.nearNetworkId;
        const validateEthash = command.validateEthash;

        const clientPk = nearlib.KeyPair.fromString(clientSk).publicKey;
        const proverPk = nearlib.KeyPair.fromString(proverSk).publicKey;

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
    }
}

exports.InitNEARContracts = InitNEARContracts;
