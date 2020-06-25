const utils = require('ethereumjs-util');
const BN = require('bn.js');
const Web3 = require('web3');
const fs = require('fs');
const nearlib = require('near-api-js');
const {
    EthProofExtractor,
    receiptFromWeb3,
    logFromWeb3,
} = require('../lib/eth-proof-extractor');
const { verifyAccount } = require('../lib/near-helpers');
const { NearMintableToken } = require('../lib/near-mintable-token');
const { RainbowConfig } = require('../lib/config');
const {
    Eth2NearClientContract,
} = require('../lib/eth2near-client-contract');
const { serialize } = require('../lib/borsh');

function sleep (ms) {
    return new Promise((resolve) => {
        setTimeout(resolve, ms);
    });
}

class TransferETHERC20ToNear {
    static async execute (command) {
        const amount = command.amount;
        const ethSenderSk = command.ethSenderSk;
        const nearReceiverAccount = command.nearReceiverAccount;

        // @ts-ignore
        const web3 = new Web3(RainbowConfig.getParam('eth-node-url'));

        let ethSenderAccount = web3.eth.accounts.privateKeyToAccount(ethSenderSk);
        web3.eth.accounts.wallet.add(ethSenderAccount);
        web3.eth.defaultAccount = ethSenderAccount.address;
        ethSenderAccount = ethSenderAccount.address;

        // Approve tokens for transfer.
        const ethERC20Contract = new web3.eth.Contract(
            // @ts-ignore
            JSON.parse(fs.readFileSync(RainbowConfig.getParam('eth-erc20-abi-path'))),
            RainbowConfig.getParam('eth-erc20-address'),
        );
        try {
            console.log('Approving token transfer.');
            await ethERC20Contract.methods.approve(RainbowConfig.getParam('eth-locker-address'),
                Number(amount)).send({
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
            // @ts-ignore
            JSON.parse(fs.readFileSync(RainbowConfig.getParam('eth-locker-abi-path'))),
            RainbowConfig.getParam('eth-locker-address'),
        );
        let lockedEvent;
        try {
            console.log('Transferring tokens from the ERC20 account to the token locker account.');
            const transaction = await ethTokenLockerContract.methods.lockToken(RainbowConfig.getParam('eth-erc20-address'), Number(amount),
                nearReceiverAccount)
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

        const nearMasterAccountId = RainbowConfig.getParam('near-master-account');
        const keyStore = new nearlib.keyStores.InMemoryKeyStore();
        await keyStore.setKey(RainbowConfig.getParam('near-network-id'), nearMasterAccountId,
            nearlib.KeyPair.fromString(RainbowConfig.getParam('near-master-sk')));
        const near = await nearlib.connect({
            nodeUrl: RainbowConfig.getParam('near-node-url'),
            networkId: RainbowConfig.getParam('near-network-id'),
            masterAccount: nearMasterAccountId,
            deps: { keyStore: keyStore },
        });
        const nearMasterAccount = new nearlib.Account(near.connection, nearMasterAccountId);
        await verifyAccount(near, nearMasterAccountId);

        const nearTokenContract = new nearlib.Contract(nearMasterAccount, RainbowConfig.getParam('near-fun-token-account'), {
            changeMethods: ['new'],
            viewMethods: ['get_balance'],
        });
        const nearTokenContractBorsh = new NearMintableToken(nearMasterAccount, RainbowConfig.getParam('near-fun-token-account'));
        await nearTokenContractBorsh.accessKeyInit();

        // Extract proof.
        const extractor = new EthProofExtractor();
        extractor.initialize(RainbowConfig.getParam('eth-node-url'));
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
                console.log(`Log ${JSON.stringify(log)}`);
                console.log(`Log entry ${logFromWeb3(log)}`);
                console.log("log_entry_data");
                console.log(log_entry_data.toString('hex'));
                
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
                console.log(`Transferring ${amount} tokens from ${lockedEvent.returnValues.token} ERC20. From ${lockedEvent.returnValues.sender} sender to ${new_owner_id} recipient`);

                const blockNumber = block.number;
                // Wait until client accepts this block number.
                const clientAccount = RainbowConfig.getParam('eth2near-client-account');
                const ethClientContract = new Eth2NearClientContract(nearMasterAccount, clientAccount);
                while (true) {
                    // @ts-ignore
                    const last_block_number = (await ethClientContract.last_block_number()).toNumber();
                    if (last_block_number < blockNumber) {
                        const delay = 10;
                        console.log(`Eth2NearClient is currently at block ${last_block_number}. Waiting for block ${blockNumber}. Sleeping for ${delay} sec.`);
                        await sleep(delay * 1000);
                    } else {
                        break;
                    }
                }

                // @ts-ignore
                const old_balance = await nearTokenContract.get_balance({
                    owner_id: new_owner_id,
                });
                console.log(`Balance of ${new_owner_id} before the transfer is ${old_balance}`);

                // @ts-ignore
                await nearTokenContractBorsh.mint(
                    proof_locker,
                    new BN('300000000000000'),
                );
                console.log(`Transferred`);

                // @ts-ignore
                const new_balance = await nearTokenContract.get_balance({
                    owner_id: new_owner_id,
                });
                console.log(`Balance of ${new_owner_id} after the transfer is ${new_balance}`);

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
        process.exit(0);
    }
}

exports.TransferETHERC20ToNear = TransferETHERC20ToNear;
