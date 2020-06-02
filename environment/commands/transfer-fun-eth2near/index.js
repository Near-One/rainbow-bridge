const utils = require('ethereumjs-util');
const BN = require('bn.js');
const Web3 = require('web3');
const fs = require('fs');
const nearlib = require('nearlib');
const {
    EthProofExtractor,
    receiptFromWeb3,
    logFromWeb3,
} = require('../../lib/eth-proof-extractor');
const { verifyAccount } = require('../../lib/near-helpers');
const {
    TokenLockerContract,
} = require('../../lib/near-locker-contract');

class TransferFunETH2NEAR {
    static async execute (command) {
        const web3 = new Web3(command.ethNodeUrl);
        let keyStore = new nearlib.keyStores.InMemoryKeyStore();
        await keyStore.setKey(command.nearNetworkId, command.nearReceiverAccount,
            nearlib.KeyPair.fromString(command.nearReceiverSk));
        const near = await nearlib.connect({
            nodeUrl: command.nearNodeUrl,
            networkId: command.nearNetworkId,
            masterAccount: command.nearReceiverAccount,
            deps: { keyStore: keyStore },
        });
        const nearReceiverAccount = new nearlib.Account(near.connection, command.nearReceiverAccount);
        await verifyAccount(near, command.nearReceiverAccount);

        let ethSenderAccount = web3.eth.accounts.privateKeyToAccount(command.ethSenderSk);
        web3.eth.accounts.wallet.add(ethSenderAccount);
        web3.eth.defaultAccount = ethSenderAccount.address;
        ethSenderAccount = (await web3.eth.getAccounts())[0];

        // Approve tokens for transfer.
        const ethERC20Contract = new web3.eth.Contract(
            JSON.parse(fs.readFileSync(command.ethTokenAbiPath)),
            command.ethTokenAddress,
        );
        try {
            console.log('Approving token transfer.');
            await ethERC20Contract.methods.approve(command.ethLockerAddress, Number(command.amount)).send({
                from: ethSenderAccount,
                gas: 5000000,
                handleRevert: true,
            });
            console.log('Approved token transfer.');
        } catch (txRevertMessage) {
            console.log('Failure.');
            console.log(txRevertMessage.toString());
            process.exit(1);
        }

        // Lock the token.
        const ethTokenLockerContract = new web3.eth.Contract(
            JSON.parse(fs.readFileSync(command.ethLockerAbiPath)),
            command.ethLockerAddress,
        );
        let lockedEvent;
        try {
            console.log('Transferring tokens from the ERC20 account to the token locker account.');
            const transaction = await ethTokenLockerContract.methods.lockToken(command.ethTokenAddress, Number(command.amount), command.nearReceiverAccount)
                .send({
                    from: ethSenderAccount,
                    gas: 5000000,
                    handleRevert: true,
                });
            lockedEvent = transaction.events.Locked;
            console.log('Success.');
        } catch (txRevertMessage) {
            console.log('Failure.');
            console.log(txRevertMessage.toString());
            process.exit(1);
        }


        let nearTokenContract = new nearlib.Contract(nearReceiverAccount, command.nearTokenAddress, {
            changeMethods: ['new'],
            viewMethods: ['get_balance'],
        });
        let nearLockerContract = new TokenLockerContract(nearReceiverAccount, command.nearLockerAddress);
        await nearLockerContract.accessKeyInit();
        // Extract proof.
        const extractor = new EthProofExtractor();
        extractor.initialize(command.ethNodeUrl);
        const receipt = await extractor.extractReceipt(lockedEvent.transactionHash);
        const block = await extractor.extractBlock(receipt.blockNumber);
        const tree = await extractor.buildTrie(block);
        const proof = await extractor.extractProof(block, tree, receipt.transactionIndex);

        let txLogIndex = -1;
        let logFound = false;
        for (const log of receipt.logs) {
            txLogIndex++;
            const blockLogIndex = log.logIndex;
            if (blockLogIndex === lockedEvent.logIndex) {
                logFound = true;
                const log_entry_data = logFromWeb3(log).serialize();
                const receipt_index = proof.txIndex;
                const receipt_data = receiptFromWeb3(receipt).serialize();
                const header_data = proof.header.serialize();
                const _proof = [];
                for (const node of proof.receiptProof) {
                    _proof.push(utils.rlp.encode(node));
                }

                const proof_locker = {
                    log_index: txLogIndex,
                    log_entry_data: log_entry_data,
                    receipt_index: receipt_index,
                    receipt_data: receipt_data,
                    header_data: header_data,
                    proof: _proof,
                };

                const new_owner_id = lockedEvent.returnValues.accountId;
                const amount = lockedEvent.returnValues.amount;

                const args_locker = {
                    token_account: nearTokenContract.contractId,
                    new_owner_id: new_owner_id,
                    amount: amount,
                    proof: proof_locker,
                };

                await nearLockerContract.unlock_token(
                    args_locker,
                    new BN('1000000000000000'),
                );
                console.log(`Transferred ${amount} tokens to ${new_owner_id}`);

                const new_balance = await nearTokenContract.get_balance({
                    owner_id: new_owner_id,
                });
                console.log(`New ${new_owner_id} balance is ${new_balance}`);

                break;
            }
        }

        if (!logFound) {
            console.log(`ERROR log not found for event ${lockedEvent}`);
        }

        try {
            // Only WebSocket provider can close.
            web3.currentProvider.connection.close();
        } catch (e) {
        }
    }
}

exports.TransferFunETH2NEAR = TransferFunETH2NEAR;
