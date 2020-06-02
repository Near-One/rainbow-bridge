const nearlib = require('nearlib');
const path = require('path');
const { maybeCreateAccount, verifyAccount } = require('../../lib/near-helpers');
const {
    TokenLockerContract,
} = require('../../lib/near-locker-contract');

class InitNEARTestContracts {
    static async execute (command) {
        const tokenAccountId = 'funtoken';
        const lockerAccountId = 'nearlocker';

        const keyStore = new nearlib.keyStores.InMemoryKeyStore();
        await keyStore.setKey(command.nearNetworkId, command.nearMasterAccount,
            nearlib.KeyPair.fromString(command.nearMasterSk));
        await keyStore.setKey(command.nearNetworkId, tokenAccountId,
            nearlib.KeyPair.fromString(command.nearMasterSk));
        await keyStore.setKey(command.nearNetworkId, lockerAccountId,
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
        const tokenInitSupply = '1000000';
        const tokenContractPath = path.join(command.contractsDir, 'fungible_token.wasm');
        console.log("Deploying fungible token");
        await maybeCreateAccount(near, tokenAccountId, tokenAccountId, tokenPK, tokenInitBalance, tokenContractPath);
        let tokenContract = new nearlib.Contract(tokenAccount, tokenAccountId, {
            changeMethods: ['new'],
            viewMethods: ['get_balance'],
        });
        try {
            // Try initializing the contract. Give it initial supply of tokens and assign to the locker contract.
            await tokenContract.new({
                owner_id: lockerAccountId,
                total_supply: tokenInitSupply,
            });
        } catch (e) {
            // I guess not
        }
        const lockerBalance = await tokenContract.get_balance({
            owner_id: lockerAccountId,
        });
        console.log(`Fungible token deployed. ${lockerBalance} tokens given to ${lockerAccountId}`);
        console.log("Fungible token address:");
        console.log(tokenAccountId);


        console.log("Deploying locker contract");
        const lockerAccount = new nearlib.Account(near.connection, lockerAccountId);
        const lockerInitBalance = '1000000000000000000000000000';
        const lockerContractPath = path.join(command.contractsDir, 'locker.wasm');
        await maybeCreateAccount(near, command.nearMasterAccount, lockerAccountId, tokenPK, lockerInitBalance, lockerContractPath);
        let lockerContract = new TokenLockerContract(lockerAccount, lockerAccount.accountId);
        await lockerContract.maybeInitialize(command.nearProverAccount, command.validateEthash !== 'true');
        console.log("Locker contract deployed to address:")
        console.log(lockerAccountId);
    }
}

exports.InitNEARTestContracts = InitNEARTestContracts;
