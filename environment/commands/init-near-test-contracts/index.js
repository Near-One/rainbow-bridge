const nearlib = require('nearlib');
const fs = require('fs');
const BN = require('bn.js');
const { maybeCreateAccount, accountExists, accountHasTheKey, verifyAccount } =
    require('../../lib/near-helpers');

class InitNEARTestContracts {
    static async execute (command) {
        const keyStore = new nearlib.keyStores.InMemoryKeyStore();
        await keyStore.setKey(command.nearNetworkId, command.nearMasterAccount,
            nearlib.KeyPair.fromString(command.nearMasterSk));
        const near = await nearlib.connect({
            nodeUrl: command.nearNodeUrl,
            networkId: command.nearNetworkId,
            masterAccount: command.nearMasterAccount,
            deps: { keyStore: keyStore },
        });

        await verifyAccount(near, command.nearMasterAccount);
    // TODO: Finish initialization of the accounts.
    }
}

exports.InitNEARTestContracts = InitNEARTestContracts;
