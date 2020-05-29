const utils = require('ethereumjs-util');
const Web3 = require('web3');
const fs = require('fs');
const path = require('path');
const BN = require('bn.js');
const { EthProofExtractor, receiptFromWeb3, logFromWeb3 } = require('../eth-proof-extractor');

class Eth2NearTransferExample {
    constructor(ethProverContract, nearLockerContract, nearTokenContract, ethNodeURL, ethMasterSK, ethContractsDir, nearUserAccount) {
        this.ethProverContract = ethProverContract;
        this.nearLockerContract = nearLockerContract;
        this.nearTokenContract = nearTokenContract;
        this.ethNodeURL = ethNodeURL;
        this.ethMasterSK = ethMasterSK;
        this.ethContractsDir = ethContractsDir;
        this.nearUserAccount = nearUserAccount;
    }

    async initialize() {
        this.web3 = new Web3(this.ethNodeURL);
        // Set master SK.
        const acc = this.web3.eth.accounts.privateKeyToAccount(this.ethMasterSK);
        this.web3.eth.accounts.wallet.add(acc);
        this.web3.eth.defaultAccount = acc.address;
        this.ethMasterAccount = (await this.web3.eth.getAccounts())[0];

        // Initialize MyERC20 contract.
        console.log("Deploying MyERC20 contract.");
        this.myERC20Contract = new this.web3.eth.Contract(
            JSON.parse(fs.readFileSync(path.join(this.ethContractsDir, 'MyERC20.full.abi')))
        );
        this.myERC20Contract = await this.myERC20Contract.deploy({
            data: '0x' + fs.readFileSync(path.join(this.ethContractsDir, 'MyERC20.full.bin'))
        }).send({
            from: this.ethMasterAccount,
            gas: 3000000,
            handleRevert: true,
        });
        console.log("Deployed MyERC20 contract.");

        // Initialize Token Locker contract.
        console.log("Deploying TokenLocker contract.");
        this.tokenLockerContract = new this.web3.eth.Contract(
            JSON.parse(fs.readFileSync(path.join(this.ethContractsDir, 'TokenLocker.full.abi')))
        );
        this.tokenLockerContract = await this.tokenLockerContract.deploy({
            data: '0x' + fs.readFileSync(path.join(this.ethContractsDir, 'TokenLocker.full.bin'))
        }).send({
            from: this.ethMasterAccount,
            gas: 3000000,
            handleRevert: true,
        });
        console.log("Deployed TokenLocker contract.");
    }

    async logRevertedTx(txRevertMessage) {
        const err = txRevertMessage.toString();
        const receipt = JSON.parse(err.substr(err.indexOf('{')));
        const tx = await this.web3.eth.getTransaction(receipt.transactionHash);
        try {
            await this.web3.eth.call(tx, tx.blockNumber);
        } catch (callRevertReason) {
            const err = callRevertReason.toString();
            console.log('Reverted! Reason:', err.substr(err.lastIndexOf(':') + 2));
        }
    }

    // Lock token in a loop.
    async run() {
        await this.subscribeToLocked();
        const lockToken = async () => {
            // First approve transfer on the token.
            try {
                console.log("Approving token transfer.");
                await this.myERC20Contract.methods.approve(this.tokenLockerContract.options.address, 1).send({
                    from: this.ethMasterAccount,
                    gas: 5000000,
                    handleRevert: true,
                });
                console.log("Approved token transfer.");
            } catch (txRevertMessage) {
                await this.logRevertedTx(txRevertMessage);
            }
            // Then transfer it from the token locker.
            try {
                console.log("Transferring tokens to the token locker.");
                const approve_tx = await this.tokenLockerContract.methods.lockToken(this.myERC20Contract.options.address, 1, this.nearUserAccount).send({
                    from: this.ethMasterAccount,
                    gas: 5000000,
                    handleRevert: true,
                });
                console.log("Transfered tokens to the token locker.");
            } catch (txRevertMessage) {
                await this.logRevertedTx(txRevertMessage);
            }
            await lockToken();
        };
        await lockToken();
    }

    // Subscribe to `Locked` event, extract the proof and submit it to the Locker contract on Near blockchain.
    async subscribeToLocked() {
        let extractor = new EthProofExtractor();
        extractor.initialize(this.ethNodeURL);
        let ethProverContract = this.ethProverContract;
        let nearTokenContract = this.nearTokenContract;
        let nearLockerContract = this.nearLockerContract;

        this.tokenLockerContract.events.Locked({
            "fromBlock": "latest"
        },
        async function(error, event) {
            console.log(event);
            const receipt = await extractor.extractReceipt(event.transactionHash);
            const block = await extractor.extractBlock(receipt.blockNumber);
            const tree = await extractor.buildTrie(block);
            const proof = await extractor.extractProof(block, tree, receipt.transactionIndex);

            let txLogIndex = -1;
            let logFound = false;
            for (const log of receipt.logs) {
                txLogIndex++;
                const blockLogIndex = log.logIndex;
                if (blockLogIndex == event.logIndex) {
                    logFound = true;
                    const log_entry_data = logFromWeb3(log).serialize();
                    const receipt_index = proof.txIndex;
                    const receipt_data = receiptFromWeb3(receipt).serialize();
                    const header_data = proof.header.serialize();
                    let _proof = [];
                    for (let node of proof.receiptProof) {
                        _proof.push(utils.rlp.encode(node));
                    }

                    const skip_bridge_call = true;

                    const args = {
                        log_index: txLogIndex,
                        log_entry_data: log_entry_data,
                        receipt_index: receipt_index,
                        receipt_data: receipt_data,
                        header_data: header_data,
                        proof: _proof,
                        skip_bridge_call: skip_bridge_call,
                    };

                    let result = await ethProverContract.verify_log_entry(
                        args,
                        new BN('1000000000000000')
                    );
                    console.log("Verified log entry");

                    const proof_locker = {
                        log_index: txLogIndex,
                        log_entry_data: log_entry_data,
                        receipt_index: receipt_index,
                        receipt_data: receipt_data,
                        header_data: header_data,
                        proof: _proof,
                    };

                    const new_owner_id = event.returnValues.accountId;
                    const amount = event.returnValues.amount;

                    const args_locker = {
                        token_account: nearTokenContract.contractId,
                        new_owner_id: new_owner_id,
                        amount: amount,
                        proof: proof_locker
                    };


                    await nearLockerContract.unlock_token(
                        args_locker,
                        new BN('1000000000000000')
                    );
                    console.log(`Transferred ${amount} tokens to ${new_owner_id}`);

                    let new_balance = await nearTokenContract.get_balance({
                        "owner_id": new_owner_id
                    });
                    console.log(`New ${new_owner_id} balance is ${new_balance}`);

                    break;
                }
            }

            if (!logFound) {
                console.log(`ERROR log not found for event ${event}`);
            }
        }
        );
    }

    disconnect() {
        if (this.web3.currentProvider.connection.close) { // Only WebSocket provider has close, HTTPS don't
            this.web3.currentProvider.connection.close();
        }
    }
}

exports.Eth2NearTransferExample = Eth2NearTransferExample;