const nearlib = require('nearlib');
const fs = require('fs');
const BN = require('bn.js');

// Check if account exists and if it does not creates it using master account. Also deploys the code and creates
// an access key.
async function maybeCreateAccount (near, masterAccountId, accountId, accountPK, initBalance, contractPath) {
    if (!await accountExists(near, accountId)) {
        console.log('Account %s does not exist creating it.', accountId);
        const masterAccount = new nearlib.Account(near.connection, masterAccountId);
        const balance = new BN(initBalance);
        try {
            await masterAccount.createAccount(accountId, accountPK, balance);
        } catch (e) {
            console.log('Failed to create account %s. ERROR: %s', accountId, e);
            process.exit(1);
        }
        console.log('Created account %s', accountId);

        const account = new nearlib.Account(near.connection, accountId);
        try {
            const data = fs.readFileSync(contractPath);
            await account.deployContract(data);
        } catch (e) {
            console.log('Failed to deploy contract to account %s. ERROR: %s', accountId, e);
            process.exit(1);
        }
        console.log('Deployed contract to account %s', accountId);
    }
}

// Checks whether the account exists.
async function accountExists (near, accountId) {
    const account = new nearlib.Account(near.connection, accountId);
    try {
        await account.fetchState();
        return true;
    } catch (e) {
        return false;
    }
}

// Checks whether the account has the key specified in the keyStore.
async function accountHasTheKey (near, accountId) {
    const account = new nearlib.Account(near.connection, accountId);
    const keyStoreKey = await near.config.keyStore.getKey(near.config.networkId, accountId);
    const keys = await account.getAccessKeys();
    const accessKey = keys.find(key => key.public_key === keyStoreKey.getPublicKey().toString());
    if (accessKey) {
        return true;
    } else {
        return false;
    }
}

// Verify that account exists and it has the key that we specified in the keyStore.
async function verifyAccount (near, accountId) {
    if (!await accountExists(near, accountId)) {
        console.log('Failed to fetch state of the %s account. Is it initialized?', accountId);
        process.exit(1);
    }

    if (!await accountHasTheKey(near, accountId)) {
        console.log('Account %s does not have the access key that can be used to operate with it.', accountId);
        process.exit(1);
    }
}

exports.maybeCreateAccount = maybeCreateAccount;
exports.accountExists = accountExists;
exports.accountHasTheKey = accountHasTheKey;
exports.verifyAccount = verifyAccount;
