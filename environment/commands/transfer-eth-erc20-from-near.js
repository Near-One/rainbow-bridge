const nearlib = require('near-api-js');
const BN = require('bn.js');
const { verifyAccount } = require('../lib/near-helpers');
const { NearMintableToken } = require('../lib/near-mintable-token');
const { RainbowConfig } = require('../lib/config');

class TransferEthERC20FromNear {
    static async execute (command) {

        const nearSenderAccountId = command.nearSenderAccount;
        const keyStore = new nearlib.keyStores.InMemoryKeyStore();
        await keyStore.setKey(RainbowConfig.getParam('near-network-id'), nearSenderAccountId,
            nearlib.KeyPair.fromString(command.nearSenderSk));
        const near = await nearlib.connect({
            nodeUrl: RainbowConfig.getParam('near-node-url'),
            networkId: RainbowConfig.getParam('near-network-id'),
            masterAccount: nearSenderAccountId,
            deps: { keyStore: keyStore },
        });
        const nearSenderAccount = new nearlib.Account(near.connection, nearSenderAccountId);
        await verifyAccount(near, nearSenderAccountId);

        const nearTokenContract = new nearlib.Contract(nearSenderAccount, RainbowConfig.getParam('near-fun-token-account'), {
            changeMethods: ['new', 'burn'],
            viewMethods: ['get_balance'],
        });
        const nearTokenContractBorsh = new NearMintableToken(nearSenderAccount, RainbowConfig.getParam('near-fun-token-account'));
        await nearTokenContractBorsh.accessKeyInit();

        // Burn the token on Near side.
        const old_balance = await nearTokenContract.get_balance({
            owner_id: command.nearSenderAccount,
        });
        console.log(`Balance of ${command.nearSenderAccount} before burning: ${old_balance}`);
        const ethReceiverAddress = command.ethReceiverAddress.startsWith('0x') ? command.ethReceiverAddress.substr(2) : command.ethReceiverAddress;
        console.log(`Burning ${command.amount} tokens on NEAR blockchain in favor of ${ethReceiverAddress}.`);
        // let txBurn = await nearTokenContract.burn({
        //     amount: command.amount,
        //     recipient: ethReceiverAddress
        // }, new BN('300000000000000'));
        let txBurn = await nearSenderAccount.functionCall(
            RainbowConfig.getParam('near-fun-token-account'),
            'burn',
            { amount: command.amount, recipient: ethReceiverAddress },
            new BN('300000000000000'),
            new BN(0)
            );
        // Either hash of the transaction or the receipt. When transaction singe is the same as the fun token address it is
        // the hash of the transaction, since Near runtime executes contract immediately. Otherwise hash of the receipt
        // that was executed on another shard.
        let tx_receipt_id;
        if (RainbowConfig.getParam('near-fun-token-account') === command.nearSenderAccount) {
            if (txBurn.receipts_outcome.length <= 1) {
                tx_receipt_id = txBurn.transaction.hash;
            } else {
                console.error(`Expected exactly one receipt when signer and fun token account are the same, but received: ${JSON.stringify(txBurn)}`);
                process.exit(1);
            }
        } else {
            if (txBurn.receipts_outcome.length <= 2) {
                let receipts = txBurn.transaction_outcome.outcome.receipt_ids;
                if (receipts.length === 1) {
                    tx_receipt_id = receipts[0];
                } else {
                    console.error(`Fungible token transaction call is expected to produce only one receipt, but produced: ${JSON.stringify(txBurn)}`);
                    process.exit(1);
                }
            } else {
                console.error(`Fungible token is not expected to perform cross contract calls: ${JSON.stringify(txBurn)}`);
                process.exit(1);
            }
        }
        // TODO: Wait for the block with the given receipt/transaction in Near2EthClient.
        console.log(`Burnt ${JSON.stringify(command.txBurn)}`);
        const new_balance = await nearTokenContract.get_balance({
            owner_id: command.nearSenderAccount,
        });
        console.log(`Balance of ${command.nearSenderAccount} after burning: ${new_balance}`);
    }
}

exports.TransferEthERC20FromNear = TransferEthERC20FromNear;