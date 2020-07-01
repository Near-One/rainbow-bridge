const Web3 = require('web3');
const nearlib = require('near-api-js');
const BN = require('bn.js');
const fs = require('fs');
const bs58 = require('bs58');
const { toBuffer } = require('eth-util-lite');
const { verifyAccount } = require('../lib/near-helpers');
const { NearMintableToken } = require('../lib/near-mintable-token');
const { RainbowConfig } = require('../lib/config');
const { borshifyOutcomeProof } = require(`../lib/borsh`);

function sleep (ms) {
    return new Promise((resolve) => {
        setTimeout(resolve, ms);
    });
}

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
        let txReceiptId;
        let txReceiptBlockHash;
        let idType;
        if (RainbowConfig.getParam('near-fun-token-account') === command.nearSenderAccount) {
            if (txBurn.receipts_outcome.length <= 1) {
                txReceiptId = txBurn.transaction.hash;
                txReceiptBlockHash = txBurn.transaction_outcome.block_hash;
                idType = 'transaction';
            } else {
                console.error(`Expected exactly one receipt when signer and fun token account are the same, but received: ${JSON.stringify(txBurn)}`);
                process.exit(1);
            }
        } else {
            if (txBurn.receipts_outcome.length <= 2) {
                let receipts = txBurn.transaction_outcome.outcome.receipt_ids;
                if (receipts.length === 1) {
                    txReceiptId = receipts[0];
                    txReceiptBlockHash = txBurn.receipts_outcome.find(el => el.id == txReceiptId).block_hash;
                    idType = 'receipt';
                } else {
                    console.error(`Fungible token transaction call is expected to produce only one receipt, but produced: ${JSON.stringify(txBurn)}`);
                    process.exit(1);
                }
            } else {
                console.error(`Fungible token is not expected to perform cross contract calls: ${JSON.stringify(txBurn)}`);
                process.exit(1);
            }
        }
        // Get block in which the outcome was processed.
        const outcomeBlock = await near.connection.provider.block({ blockId: txReceiptBlockHash});
        const outcomeBlockHeight = new BN(outcomeBlock.header.height);

        // Wait for the block with the given receipt/transaction in Near2EthClient.
        let web3 = new Web3(RainbowConfig.getParam('eth-node-url'));
        let ethMasterAccount = web3.eth.accounts.privateKeyToAccount(RainbowConfig.getParam('eth-master-sk'));
        web3.eth.accounts.wallet.add(ethMasterAccount);
        web3.eth.defaultAccount = ethMasterAccount.address;
        ethMasterAccount = ethMasterAccount.address;
        let clientContract = new web3.eth.Contract(
            // @ts-ignore
            JSON.parse(fs.readFileSync(RainbowConfig.getParam('near2eth-client-abi-path'))),
            RainbowConfig.getParam('near2eth-client-address'), {
                from: ethMasterAccount,
                handleRevert: true,
            },
        );

        let clientBlock;
        let clientBlockHeight;
        let clientBlockValidAfter;
        let clientBlockHashB58;
        let clientBlockHashHex;
        let clientBlockMerkleRoot;
        while (true) {
            clientBlock = await clientContract.methods.last().call();
            clientBlockHeight = new BN(clientBlock.height);
            clientBlockValidAfter = new BN(clientBlock.validAfter);
            clientBlockHashHex = await clientContract.methods.blockHashes(clientBlockHeight).call();
            clientBlockMerkleRoot = await clientContract.methods.blockMerkleRoots(clientBlockHeight).call();
            clientBlockHashB58 = bs58.encode(toBuffer(clientBlockHashHex));
            console.log(`Current light client head is: hash=${clientBlockHashB58}, height=${clientBlockHeight.toString()}`);

            const chainBlock = await web3.eth.getBlock('latest');
            const chainBlockTimestamp = new BN(chainBlock.timestamp);
            if (clientBlockHeight.gt(outcomeBlockHeight)) {
                console.log(`Near2EthClient block is at ${clientBlockHeight.toString()} which is further than the needed block ${outcomeBlockHeight.toString()}`);
                break;
            } else if (chainBlockTimestamp.gte(clientBlockValidAfter) && clientBlockHeight.eq(outcomeBlockHeight)) {
                console.log(`Near2EthClient block is at ${clientBlockHeight.toString()} which is the block of the outcome. And the light client block is valid.`);
                break;
            } else if (chainBlockTimestamp.lt(clientBlockValidAfter) && clientBlockHeight.eq(outcomeBlockHeight)) {
                const sleepSec = clientBlockValidAfter.sub(chainBlockTimestamp).toNumber();
                console.log(`Block ${clientBlockHeight.toString()} is not valid yet. Sleeping ${sleepSec} seconds.`);
                await sleep(sleepSec * 1000);
            } else {
                const sleepSec = 10;
                console.log(`Block ${outcomeBlockHeight.toString()} is not available on the light client yet. Current height of light client is ${clientBlockHeight.toString()}. Sleeping ${sleepSec} seconds.`);
                await sleep(sleepSec * 1000);
            }
        }

        console.log(`Burnt ${JSON.stringify(command.amount)}`);
        const new_balance = await nearTokenContract.get_balance({
            owner_id: command.nearSenderAccount,
        });
        console.log(`Balance of ${command.nearSenderAccount} after burning: ${new_balance}`);

        // Get the outcome proof only use block merkle root that we know is available on the Near2EthClient.
        let proofRes;
        if (idType === 'transaction') {
            proofRes = await near.connection.provider.sendJsonRpc('light_client_proof', {
                type: 'transaction',
                transaction_hash: txReceiptId,
                // TODO: Use proper sender.
                receiver_id: command.nearSenderAccount,
                light_client_head: clientBlockHashB58
            });
        } else if (idType === 'receipt') {
            proofRes = await near.connection.provider.sendJsonRpc('light_client_proof', {
                type: 'receipt',
                receipt_id: txReceiptId,
                // TODO: Use proper sender.
                receiver_id: command.nearSenderAccount,
                light_client_head: clientBlockHashB58
            });
        } else {
            console.error('Unreachable');
            process.exit(1);
        }

        // Check that the proof is correct.
        let proverContract = new web3.eth.Contract(
            // @ts-ignore
            JSON.parse(fs.readFileSync(RainbowConfig.getParam('near2eth-prover-abi-path'))),
            RainbowConfig.getParam('near2eth-prover-address'), {
                from: ethMasterAccount,
                handleRevert: true,
            },
        );
        const borshProofRes = borshifyOutcomeProof(proofRes);
        console.log(`proof: ${JSON.stringify(proofRes)}`);
        console.log(`client height: ${clientBlockHeight.toString()}`);
        console.log(`root: ${clientBlockMerkleRoot}`);
        let proverRes = await proverContract.methods.proveOutcome(borshProofRes, clientBlockHeight).call();

        const ethTokenLockerContract = new web3.eth.Contract(
            // @ts-ignore
            JSON.parse(fs.readFileSync(RainbowConfig.getParam('eth-locker-abi-path'))),
            RainbowConfig.getParam('eth-locker-address'), {
                from: ethMasterAccount,
                handleRevert: true,
            },
        );

        const ethERC20Contract = new web3.eth.Contract(
            // @ts-ignore
            JSON.parse(fs.readFileSync(RainbowConfig.getParam('eth-erc20-abi-path'))),
            RainbowConfig.getParam('eth-erc20-address'), {
                from: ethMasterAccount,
                handleRevert: true,
            },
        );

        const oldBalance = await ethERC20Contract.methods.balanceOf(command.ethReceiverAddress).call();
        console.log(`ERC20 balance of ${command.ethReceiverAddress} before the transfer: ${oldBalance}`);
        const txUnlock = await ethTokenLockerContract.methods.unlockToken(
            borshProofRes, clientBlockHeight
        ).send({
            from: ethMasterAccount,
            gas: 5000000,
            handleRevert: true
        });
        const totalSupply = await ethERC20Contract.methods.totalSupply().call();
        const newBalance = await ethERC20Contract.methods.balanceOf(command.ethReceiverAddress).call();
        console.log(`ERC20 balance of ${command.ethReceiverAddress} after the transfer: ${newBalance}`);
        process.exit(0);
    }
}

exports.TransferEthERC20FromNear = TransferEthERC20FromNear;