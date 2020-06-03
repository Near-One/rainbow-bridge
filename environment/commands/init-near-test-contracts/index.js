const nearlib = require('nearlib');
const path = require('path');
const { maybeCreateAccount, verifyAccount } = require('../../lib/near-helpers');

class InitNEARTestContracts {
    static async execute (command) {
        const tokenAccountId = 'funtoken';

        const keyStore = new nearlib.keyStores.InMemoryKeyStore();
        await keyStore.setKey(command.nearNetworkId, command.nearMasterAccount,
            nearlib.KeyPair.fromString(command.nearMasterSk));
        await keyStore.setKey(command.nearNetworkId, tokenAccountId,
            nearlib.KeyPair.fromString(command.nearMasterSk));
        const near = await nearlib.connect({
            nodeUrl: command.nearNodeUrl,
            networkId: command.nearNetworkId,
            masterAccount: command.nearMasterAccount,
            deps: { keyStore: keyStore },
        });

        const masterPK = nearlib.KeyPair.fromString(command.nearMasterSk).publicKey;
        await verifyAccount(near, command.nearMasterAccount);

        const tokenPK = masterPK;
        const tokenAccount = new nearlib.Account(near.connection, tokenAccountId);
        const tokenInitBalance = '1000000000000000000000000000';
        const tokenContractPath = path.join(command.contractsDir, 'fungible_token.wasm');
        console.log("Deploying fungible token");
        await maybeCreateAccount(near, command.nearMasterAccount, tokenAccountId, tokenPK, tokenInitBalance, tokenContractPath);
        let tokenContract = new nearlib.Contract(tokenAccount, tokenAccountId, {
            changeMethods: ['new'],
            viewMethods: ['get_balance'],
        });
        try {
            // Try initializing the contract. Give it initial supply of tokens and assign to the locker contract.
            await tokenContract.new({
                // Give 0 tokens to itself.
                owner_id: tokenAccountId,
                total_supply: '0',
                prover_account: command.nearProverAccount,
                verify_ethash: command.validateEthash === 'true'
            });
        } catch (e) {
            // I guess not
        }

        console.log(`Fungible token deployed`);
        console.log("Fungible token address:");
        console.log(tokenAccountId);
    }
}

exports.InitNEARTestContracts = InitNEARTestContracts;
