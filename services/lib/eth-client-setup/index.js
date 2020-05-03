const nearlib = require('nearlib');
const fs = require('fs');
const BN = require('bn.js');
const {EthClientContract} = require('../eth-client-contract');

class EthClientSetup {
    constructor() {
    }
    async initialize() {
        this.nearNodeURL = process.env.NEAR_NODE_URL;
        this.nearNodeNetworkId = process.env.NEAR_NODE_NETWORK_ID;

        this.masterAccountId = process.env.MASTER_ACC_ID;
        this.masterAccountSK = process.env.MASTER_SK;

        this.ethClientAccId = process.env.ETH_CLIENT_ACC_ID;
        this.ethClientSK = process.env.ETH_CLIENT_SK;
        this.ethClientPK = nearlib.KeyPair.fromString(this.ethClientSK).publicKey;
        this.ethClientInitBalance = process.env.ETH_CLIENT_INIT_BALANCE;
        this.ethClientContractPath = process.env.ETH_CLIENT_CONTRACT_PATH;

        this.ethProverAccId = process.env.ETH_PROVER_ACC_ID;
        this.ethProverSK = process.env.ETH_PROVER_SK;
        this.ethProverPK = nearlib.KeyPair.fromString(this.ethProverSK).publicKey;
        this.ethProverInitBalance = process.env.ETH_PROVER_INIT_BALANCE;
        this.ethProverContractPath = process.env.ETH_PROVER_CONTRACT_PATH;

        this.keyStore = new nearlib.keyStores.InMemoryKeyStore();
        await this.keyStore.setKey(this.nearNodeNetworkId, this.masterAccountId, nearlib.KeyPair.fromString(this.masterAccountSK));
        await this.keyStore.setKey(this.nearNodeNetworkId, this.ethClientAccId, nearlib.KeyPair.fromString(this.ethClientSK));
        await this.keyStore.setKey(this.nearNodeNetworkId, this.ethProverAccId, nearlib.KeyPair.fromString(this.ethProverSK));
        this.near = await nearlib.connect({
            nodeUrl: this.nearNodeURL,
            networkId: this.nearNodeNetworkId,
            masterAccount: this.masterAccountId,
            deps: {
                keyStore: this.keyStore
            }
        });

        // const masterAccount = new nearlib.Account(this.near.connection, this.masterAccountId);
        // const balance = new BN('1000000000000000000000000000');
        // await masterAccount.createAccount('foobarbar6', this.ethClientPK, balance);


        // Initialize accounts and deploy the contracts. Call initialization functions if needed.
        await this.verifyAccount(this.masterAccountId);
        // await this.maybeCreateAccount('foobarbar7', this.ethClientPK, this.ethClientInitBalance, this.ethClientContractPath);

        await this.maybeCreateAccount(this.ethClientAccId, this.ethClientPK, this.ethClientInitBalance, this.ethClientContractPath);
        await this.verifyAccount(this.ethClientAccId);
        this.ethClientAccount = new nearlib.Account(this.near.connection, this.ethClientAccId);
        this.ethClientContract = new EthClientContract(this.ethClientAccount);
        await this.ethClientContract.maybeInitialize(false);

        await this.maybeCreateAccount(this.ethProverAccId, this.ethProverPK, this.ethProverInitBalance, this.ethProverContractPath);
        await this.maybeCreateAccount(this.ethProverAccId, this.ethClientPK, this.ethClientInitBalance, this.ethClientContractPath);
    }

    // Check if account exists and if it does not creates it using master account. Also deploys the code and creates
    // an access key.
    async maybeCreateAccount(accountId, accountPK, initBalance, contractPath) {
        if (!await this.accountExists(accountId)) {
            console.log("Account %s does not exist creating it.", accountId);
            const masterAccount = new nearlib.Account(this.near.connection, this.masterAccountId);
            const balance = new BN(initBalance);
            try {
                await masterAccount.createAccount(accountId, accountPK, balance);
            } catch (e) {
                console.log("Failed to create account %s. ERROR: %s", accountId, e);
                process.exit(1);
            }
            console.log("Created account %s", accountId);

            const account = new nearlib.Account(this.near.connection, accountId);
            try {
                const data = fs.readFileSync(contractPath);
                await account.deployContract(data);
            } catch (e) {
                console.log("Failed to deploy contract to account %s. ERROR: %s", accountId, e);
                process.exit(1);
            }
            console.log("Deployed contract to account %s", accountId);
        }
    }

    // Checks whether the account exists.
    async accountExists(accountId) {
        const account = new nearlib.Account(this.near.connection, accountId);
        try {
            await account.fetchState();
            return true;
        } catch (e) {
            return false;
        }
    }

    // Checks whether the account has the key specified in the keyStore.
    async accountHasTheKey(accountId) {
        const account = new nearlib.Account(this.near.connection, accountId);
        const keyStoreKey = await this.keyStore.getKey(this.nearNodeNetworkId, accountId);
        const keys = await account.getAccessKeys();
        const accessKey = keys.find(key => key.public_key === keyStoreKey.getPublicKey().toString());
        if (accessKey) {
            return true;
        } else {
            return false;
        }
    }

    // Verify that account exists and it has the key that we specified in the keyStore.
    async verifyAccount(accountId) {
        if (!await this.accountExists(accountId)) {
            console.log("Failed to fetch state of the %s account. Is it initialized?", accountId);
            process.exit(1);
        }

        if (!await this.accountHasTheKey(accountId)) {
            console.log("Account %s does not have the access key that can be used to operate with it.", accountId);
            process.exit(1);
        }
    }
}

exports.EthClientSetup = EthClientSetup;