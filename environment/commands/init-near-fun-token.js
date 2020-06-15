const nearlib = require('nearlib');
const { maybeCreateAccount, verifyAccount } = require('../lib/near-helpers');
const { RainbowConfig } = require('../lib/config');

class InitNEARFunToken {
    static async execute () {
        const masterAccount = RainbowConfig.getParam('near-master-account');
        const masterSk = RainbowConfig.getParam('near-master-sk');
        const tokenAccount = RainbowConfig.getParam('near-fun-token-account');
        let tokenSk = RainbowConfig.maybeGetParam('near-fun-token-sk');
        if (!tokenSk) {
            console.log(`Secret key for fungible token is not specified. Reusing master secret key.`);
            tokenSk = masterSk;
            RainbowConfig.setParam('near-fun-token-sk', tokenSk);
        }
        const tokenContractPath = RainbowConfig.getParam('near-fun-token-contract-path');
        const tokenInitBalance = RainbowConfig.getParam('near-fun-token-init-balance');
        const proverAccount = RainbowConfig.getParam('eth2near-prover-account');

        const nearNodeUrl = RainbowConfig.getParam('near-node-url');
        const nearNetworkId = RainbowConfig.getParam('near-network-id');

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
        const tokenContract = new nearlib.Contract(new nearlib.Account(near.connection, tokenAccount), tokenAccount, {
            changeMethods: ['new'],
            viewMethods: ['get_balance'],
        });
        try {
            // Try initializing the contract. Give it no initial tokens, since they will all be
            // minted using proofs. It is irrelevant what `owner_id` is since we have no initial tokens.
            await tokenContract.new({
                // Give 0 tokens to itself.
                owner_id: tokenAccount,
                total_supply: '0',
                prover_account: proverAccount,
            });
        } catch (err) {
            console.log(`Failed to initialize the token contract ${err}`);
        }
        console.log('Fungible token deployed');
        RainbowConfig.saveConfig();
    }
}

exports.InitNEARFunToken = InitNEARFunToken;
