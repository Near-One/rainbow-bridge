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
const { sleep } = require('../lib/robust');
const { normalizeEthKey } = require('../lib/robust');

let initialCmd;

class TransferETHERC20ToNear {
    static showRetryAndExit() {
        console.log('Retry with command:');
        console.log(initialCmd);
        process.exit(1);
    }

    static async approve({ ethERC20Contract, amount, ethSenderAccount }) {
        // Approve tokens for transfer.
        try {
            console.log('Approving token transfer.');
            await ethERC20Contract.methods.approve(RainbowConfig.getParam('eth-locker-address'),
                Number(amount)).send({
                    from: ethSenderAccount,
                    gas: 5000000,
                    handleRevert: true,
                });
            console.log('Approved token transfer.');
            TransferETHERC20ToNear.recordTransferLog({ finished: 'approve' })
        } catch (txRevertMessage) {
            console.log('Failed to approve.');
            console.log(txRevertMessage.toString());
            TransferETHERC20ToNear.showRetryAndExit();
        }
    }

    static async lock({ ethTokenLockerContract, amount, nearReceiverAccount, ethSenderAccount }) {
        try {
            console.log('Transferring tokens from the ERC20 account to the token locker account.');
            const transaction = await ethTokenLockerContract.methods.lockToken(Number(amount),
                nearReceiverAccount)
                .send({
                    from: ethSenderAccount,
                    gas: 5000000,
                    handleRevert: true,
                });
            const lockedEvent = transaction.events.Locked;
            console.log('Success tranfer to locker');
            TransferETHERC20ToNear.recordTransferLog({ finished: 'lock', lockedEvent })
        } catch (txRevertMessage) {
            console.log('Failed to lock account.');
            console.log(txRevertMessage.toString());
            TransferETHERC20ToNear.showRetryAndExit();
        }
    }

    static async findProof({ extractor, lockedEvent, web3 }) {
        const receipt = await extractor.extractReceipt(lockedEvent.transactionHash);
        const block = await extractor.extractBlock(receipt.blockNumber);
        const tree = await extractor.buildTrie(block);
        const proof = await extractor.extractProof(web3, block, tree, receipt.transactionIndex);
        let txLogIndex = -1;

        let logFound = false;
        let log;
        for (let receiptLog of receipt.logs) {
            txLogIndex++;
            const blockLogIndex = receiptLog.logIndex;
            if (blockLogIndex === lockedEvent.logIndex) {
                logFound = true;
                log = receiptLog;
                break;
            }
        }
        if (logFound) {
            TransferETHERC20ToNear.recordTransferLog({ finished: 'find-proof', proof, log, txLogIndex, receipt, lockedEvent, block })
        } else {
            console.log(`Failed to find log for event ${lockedEvent}`);
            TransferETHERC20ToNear.showRetryAndExit();
        }
    }

    static async waitBlockSafe({ log, proof, receipt, txLogIndex, lockedEvent, block, ethClientContract }) {
        const log_entry_data = logFromWeb3(log).serialize();
        const receipt_index = proof.txIndex;
        const receipt_data = receiptFromWeb3(receipt).serialize();
        const header_data = proof.header_rlp;
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
        while (true) {
            // @ts-ignore
            const last_block_number = (await ethClientContract.last_block_number()).toNumber();
            const is_safe = await ethClientContract.block_hash_safe(blockNumber);
            if (!is_safe) {
                const delay = 10;
                console.log(`Eth2NearClient is currently at block ${last_block_number}. Waiting for block ${blockNumber} to be confirmed. Sleeping for ${delay} sec.`);
                await sleep(delay * 1000);
            } else {
                break;
            }
        }
        TransferETHERC20ToNear.recordTransferLog({ finished: 'block-safe', proof_locker, new_owner_id })
    }

    static async mint({ proof_locker, nearTokenContract, nearTokenContractBorsh, new_owner_id }) {
        // @ts-ignore
        const old_balance = await nearTokenContract.get_balance({
            owner_id: new_owner_id,
        });
        console.log(`Balance of ${new_owner_id} before the transfer is ${old_balance}`);
        // @ts-ignore
        try {
            await nearTokenContractBorsh.mint(
                proof_locker,
                new BN('300000000000000'),
                // We need to attach tokens because minting increases the contract state, by <600 bytes, which
                // requires an additional 0.06 NEAR to be deposited to the account for state staking.
                // Note technically 0.0537 NEAR should be enough, but we round it up to stay on the safe side.
                (new BN('100000000000000000000')).mul(new BN('600')),
            );
            console.log('Transferred');
        } catch (e) {
            console.log('Mint failed with error:');
            console.log(e);
            TransferETHERC20ToNear.showRetryAndExit();
        }

        // @ts-ignore
        const new_balance = await nearTokenContract.get_balance({
            owner_id: new_owner_id,
        });
        console.log(`Balance of ${new_owner_id} after the transfer is ${new_balance}`);
        TransferETHERC20ToNear.deleteTransferLog();
    }

    static recordTransferLog(obj) {
        fs.writeFileSync('transfer-eth-erc20-to-near.log.json', JSON.stringify(obj));
    }

    static parseBuffer(obj) {
        for (let i in obj) {
            if (obj[i] && obj[i].type === 'Buffer') {
                obj[i] = Buffer.from(obj[i].data);
            } else if (obj[i] && typeof (obj[i]) === 'object') {
                obj[i] = TransferETHERC20ToNear.parseBuffer(obj[i])
            }
        }
        return obj;
    }

    static loadTransferLog() {
        try {
            let log = JSON.parse(fs.readFileSync('transfer-eth-erc20-to-near.log.json').toString()) || {};
            return TransferETHERC20ToNear.parseBuffer(log);
        } catch (e) {
            return {};
        }
    }

    static deleteTransferLog() {
        try {
            fs.unlinkSync('transfer-eth-erc20-to-near.log.json');
        } catch (e) {
            console.log('Warning: failed to remove tranfer log');
        }
    }

    static async execute(command) {
        initialCmd = command.parent.rawArgs.join(' ');
        let transferLog = TransferETHERC20ToNear.loadTransferLog();
        const amount = command.amount;
        const ethSenderSk = command.ethSenderSk;
        const nearReceiverAccount = command.nearReceiverAccount;

        // @ts-ignore
        const web3 = new Web3(RainbowConfig.getParam('eth-node-url'));

        let ethSenderAccount = web3.eth.accounts.privateKeyToAccount(normalizeEthKey(ethSenderSk));
        web3.eth.accounts.wallet.add(ethSenderAccount);
        web3.eth.defaultAccount = ethSenderAccount.address;
        ethSenderAccount = ethSenderAccount.address;

        const ethERC20Contract = new web3.eth.Contract(
            // @ts-ignore
            JSON.parse(fs.readFileSync(RainbowConfig.getParam('eth-erc20-abi-path'))),
            RainbowConfig.getParam('eth-erc20-address'),
        );

        const nearMasterAccountId = RainbowConfig.getParam('near-master-account');
        // @ts-ignore
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

        const extractor = new EthProofExtractor();
        extractor.initialize(RainbowConfig.getParam('eth-node-url'));

        const ethTokenLockerContract = new web3.eth.Contract(
            // @ts-ignore
            JSON.parse(fs.readFileSync(RainbowConfig.getParam('eth-locker-abi-path'))),
            RainbowConfig.getParam('eth-locker-address'),
        );

        const clientAccount = RainbowConfig.getParam('eth2near-client-account');
        const ethClientContract = new Eth2NearClientContract(nearMasterAccount, clientAccount);

        if (transferLog.finished === undefined) {
            await TransferETHERC20ToNear.approve({ ethERC20Contract, amount, ethSenderAccount });
            transferLog = TransferETHERC20ToNear.loadTransferLog();
        }
        if (transferLog.finished === 'approve') {
            await TransferETHERC20ToNear.lock({ ethTokenLockerContract, amount, nearReceiverAccount, ethSenderAccount });
            transferLog = TransferETHERC20ToNear.loadTransferLog();
        }
        if (transferLog.finished === 'lock') {
            await TransferETHERC20ToNear.findProof({ extractor, lockedEvent: transferLog.lockedEvent, web3 });
            transferLog = TransferETHERC20ToNear.loadTransferLog();
        }
        if (transferLog.finished === 'find-proof') {
            await TransferETHERC20ToNear.waitBlockSafe({ ethClientContract, ...transferLog });
            transferLog = TransferETHERC20ToNear.loadTransferLog();
        }
        if (transferLog.finished === 'block-safe') {
            await TransferETHERC20ToNear.mint({ nearTokenContract, nearTokenContractBorsh, ...transferLog });
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
