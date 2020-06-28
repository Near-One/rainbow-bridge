const Web3 = require('web3');
const nearlib = require('near-api-js');
const BN = require('bn.js');
const fs = require('fs');
const bs58 = require('bs58');
const { toBuffer } = require('eth-util-lite');
const { verifyAccount } = require('../lib/near-helpers');
const { NearMintableToken } = require('../lib/near-mintable-token');
const { RainbowConfig } = require('../lib/config');

function sleep (ms) {
    return new Promise((resolve) => {
        setTimeout(resolve, ms);
    });
}

function borshifyOutcomeProof(proof) {
    const statusToBuffer = status => {
        console.log(status.SuccessValue);
        if ('SuccessValue' in status) {
            return Buffer.concat([
                Buffer.from([2]),
                Buffer.from([]),
            ]);
        } else if ('SuccessReceiptId' in status) {
            return Buffer.concat([
                Buffer.from([3]),
                bs58.decode(status.SuccessReceiptId),
            ]);
        } else {
            throw new Error("status not supported");
        }
    };
    return Buffer.concat([
        // outcome_proof
        Web3.utils.toBN(proof.outcome_proof.proof.length).toBuffer('le', 4),
        Buffer.concat(
            // outcome_proof.proof
            proof.outcome_proof.proof.map(
                p => Buffer.concat([
                    bs58.decode(p.hash),
                    Buffer.from([p.direction === 'Right' ? 1 : 0]),
                ])
            )
        ),

        // outcome_proof.block_hash
        bs58.decode(proof.outcome_proof.block_hash),

        // outcome_proof.id
        bs58.decode(proof.outcome_proof.id),

        // outcome_proof.outcome
        Buffer.concat([
            // outcome_proof.outcome.logs
            Web3.utils.toBN(proof.outcome_proof.outcome.logs.length).toBuffer('le', 4),
            // TODO: find "logs" example to serialize

            // outcome_proof.outcome.receipt_ids
            Web3.utils.toBN(proof.outcome_proof.outcome.receipt_ids.length).toBuffer('le', 4),
            Buffer.concat(
                proof.outcome_proof.outcome.receipt_ids.map(
                    r => bs58.decode(r)
                )
            ),

            // outcome_proof.outcome.gas_burnt
            Web3.utils.toBN(proof.outcome_proof.outcome.gas_burnt).toBuffer('le', 8),

            statusToBuffer(proof.outcome_proof.outcome.status),
            // outcome_proof.outcome.status.SuccessReceiptId
            // Buffer.from([3]), // TODO: support other status types
            // bs58.decode(proof.outcome_proof.outcome.status.SuccessReceiptId),

            // outcome_root_proof
            Web3.utils.toBN(0).toBuffer('le', 4),

            // block_header_lite
            bs58.decode(proof.block_header_lite.prev_block_hash),
            bs58.decode(proof.block_header_lite.inner_rest_hash),
            Web3.utils.toBN(proof.block_header_lite.inner_lite.height).toBuffer('le', 8),
            bs58.decode(proof.block_header_lite.inner_lite.epoch_id),
            bs58.decode(proof.block_header_lite.inner_lite.next_epoch_id),
            bs58.decode(proof.block_header_lite.inner_lite.prev_state_root),
            bs58.decode(proof.block_header_lite.inner_lite.outcome_root),
            Web3.utils.toBN(proof.block_header_lite.inner_lite.timestamp).toBuffer('le', 8),
            bs58.decode(proof.block_header_lite.inner_lite.next_bp_hash),
            bs58.decode(proof.block_header_lite.inner_lite.block_merkle_root),

            // block_proof
            Web3.utils.toBN(proof.block_proof.length).toBuffer('le', 4),
            Buffer.concat(
                proof.block_proof.map(
                    bp => Buffer.concat([
                        bs58.decode(bp.hash),
                        Buffer.from([bp.direction === 'Right' ? 1 : 0]),
                    ])
                )
            ),
        ])
    ]);
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
        let ethReceiverAccount = web3.eth.accounts.privateKeyToAccount(command.ethReceiverAddress);
        web3.eth.accounts.wallet.add(ethReceiverAccount);
        web3.eth.defaultAccount = ethReceiverAccount.address;
        ethReceiverAccount = ethReceiverAccount.address;
        let clientContract = new web3.eth.Contract(
            // @ts-ignore
            JSON.parse(fs.readFileSync(RainbowConfig.getParam('near2eth-client-abi-path'))),
            RainbowConfig.getParam('near2eth-client-address'), {
                from: ethReceiverAccount,
                handleRevert: true,
            },
        );

        let lastClientBlock;
        let lastClientBlockHeight;
        let clientBlockHash;
        let clientBlockHashHex;
        let blockMerkleRoot;
        while (true) {
            lastClientBlock = await clientContract.methods.last().call();
            const clientBlockHeight = lastClientBlock.height;
            clientBlockHashHex = await clientContract.methods.blockHashes(clientBlockHeight).call();
            blockMerkleRoot = await clientContract.methods.blockMerkleRoots(clientBlockHeight).call();
            clientBlockHash = bs58.encode(toBuffer(clientBlockHashHex));
            console.log(`Current light client head is: hash=${clientBlockHash}, height=${clientBlockHeight}`);
            const latestBlock = await web3.eth.getBlock('latest');
            const latestBlockTimestamp = new BN(latestBlock.timestamp);
            lastClientBlockHeight = new BN(lastClientBlock.height);
            const lastClientBlockValidAfter = new BN(lastClientBlock.validAfter);
            if (lastClientBlockHeight > outcomeBlockHeight) {
                console.log(`Near2EthClient block is at ${lastClientBlockHeight} which is further than the needed block ${outcomeBlockHeight}`);
                break;
            } else if (latestBlockTimestamp >= lastClientBlockValidAfter && lastClientBlockHeight == outcomeBlockHeight) {
                console.log(`Near2EthClient block is at ${lastClientBlockHeight} which is the block of the outcome. And the light client block is valid.`);
                break;
            } else if (latestBlockTimestamp < lastClientBlockValidAfter && lastClientBlockHeight == outcomeBlockHeight){
                const sleepSec = lastClientBlockValidAfter - latestBlockTimestamp;
                console.log(`Block ${lastClientBlockHeight} is not valid yet. Sleeping ${sleepSec} seconds.`);
                await sleep(sleepSec * 1000);
            } else {
                const sleepSec = 10;
                console.log(`Block ${outcomeBlockHeight} is not available on the light client yet. Current height of light client is ${lastClientBlockHeight}. Sleeping ${sleepSec} seconds.`);
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
                light_client_head: clientBlockHash
            });
        } else if (idType === 'receipt') {
            proofRes = await near.connection.provider.sendJsonRpc('light_client_proof', {
                type: 'receipt',
                receipt_id: txReceiptId,
                // TODO: Use proper sender.
                receiver_id: command.nearSenderAccount,
                light_client_head: clientBlockHash
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
                from: ethReceiverAccount,
                handleRevert: true,
            },
        );
        const borshProofRes = borshifyOutcomeProof(proofRes);
        console.log(JSON.stringify(proofRes));
        console.log(lastClientBlockHeight.toString());
        console.log(`hash: ${clientBlockHashHex}`);
        console.log(`root: ${blockMerkleRoot}`);
        let proverRes = await proverContract.methods.proveOutcome(borshProofRes, lastClientBlockHeight).call();
        console.log("Hello");
    }
}

exports.TransferEthERC20FromNear = TransferEthERC20FromNear;