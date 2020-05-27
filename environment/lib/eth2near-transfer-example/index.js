const Web3 = require('web3');
const nearlib = require('nearlib');
const fs = require('fs');
const path = require('path');

class Eth2NearTransferExample {
    constructor(ethNodeURL, ethMasterSK, ethContractsDir, nearLockerAccount) {
        this.ethNodeURL = ethNodeURL;
        this.ethMasterSK = ethMasterSK;
        this.ethContractsDir = ethContractsDir;
        this.nearLockerAccount = nearLockerAccount;
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
        this.subscribeToLocked();
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
                const approve_tx = await this.tokenLockerContract.methods.lockToken(this.myERC20Contract.options.address, 1, this.nearLockerAccount).send({
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
    subscribeToLocked() {
        this.tokenLockerContract.events.Locked({
            "fromBlock": "latest"
        },
        function(error, event) {
            console.log(event);
            // TODO: For non-ganache networks extract the proof and submit it to the Locker contract on Near blockchain.
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