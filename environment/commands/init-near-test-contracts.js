const nearlib = require('nearlib');
const { maybeCreateAccount, verifyAccount } = require('../lib/near-helpers');

class InitNEARTestContracts {
    static async execute (command) {
        const masterAccount = command.masterAccount;
        const masterSk = command.masterSk;
        const tokenAccount = command.tokenAccount;
        let tokenSk = command.tokenSk;
        if (!tokenSk) {
            tokenSk = masterSk;
        }
        const tokenContractPath = command.tokenContractPath;
        const tokenInitBalance = command.tokenInitBalance;
        const proverAccount = command.proverAccount;

        const nearNodeUrl = command.nearNodeUrl;
        const nearNetworkId = command.nearNetworkId;

        const tokenPk = nearlib.KeyPair.fromString(tokenSk).publicKey;

        const keyStore = new nearlib.keyStores.InMemoryKeyStore();
        await keyStore.setKey(nearNetworkId, masterAccount, nearlib.KeyPair.fromString(masterSk));
        await keyStore.setKey(nearNetworkId, tokenAccount, nearlib.KeyPair.fromString(tokenSk));
        const near = await nearlib.connect({
            nodeUrl: nearNodeUrl,
            networkId: nearNetworkId,
            masterAccount: masterAccount,
            deps: { keyStore: keyStore },
        });

        await verifyAccount(near, masterAccount);
        console.log('Deploying token contract.');
        await maybeCreateAccount(near, masterAccount, tokenAccount, tokenPk, tokenInitBalance, tokenContractPath);
        const tokenContract = new nearlib.Contract(tokenAccount, tokenAccount, {
            changeMethods: ['new'],
            viewMethods: ['get_balance'],
        });
        try {
            // Try initializing the contract. Give it initial supply of tokens and
            // assign to the locker contract.
            await tokenContract.new({
                // Give 0 tokens to itself.
                owner_id: tokenAccount,
                total_supply: '0',
                prover_account: proverAccount,
                verify_ethash: true,
            });
        } catch (err) {
            console.log(`Failed to initialize the token contract ${err}`);
        }
        console.log('Fungible token deployed');
    }
}

exports.InitNEARTestContracts = InitNEARTestContracts;
