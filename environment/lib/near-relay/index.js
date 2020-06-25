const Web3 = require('web3');
const nearlib = require('near-api-js');
const fs = require('fs');
const path = require('path');
const bs58 = require('bs58');

class NearRelay {
    constructor (ethNodeURL, nearNodeURL, nearNodeNetworkId, masterSK, nearClientContractPath) {
        this.ethNodeURL = ethNodeURL;
        this.nearNodeURL = nearNodeURL;
        this.nearNodeNetworkId = nearNodeNetworkId;
        this.masterSK = masterSK;
        this.nearClientContractPath = nearClientContractPath;
    }

    async initialize (shouldDeploy) {
        // @ts-ignore
        this.web3 = new Web3(this.ethNodeURL);
        this.near = await nearlib.connect({
            nodeUrl: this.nearNodeURL,
            networkId: this.nearNodeNetworkId,
        });

        // Set master SK.
        const acc = this.web3.eth.accounts.privateKeyToAccount(this.masterSK);
        this.web3.eth.accounts.wallet.add(acc);
        this.web3.eth.defaultAccount = acc.address;
        this.clientAccount = (await this.web3.eth.getAccounts())[0];

        // Initialize contract interface.
        this.nearClientContract = new this.web3.eth.Contract(
            // @ts-ignore
            JSON.parse(fs.readFileSync(path.join(this.nearClientContractPath, 'NearBridge.full.abi'))),
            process.env.NEAR_BRIDGE_SMART_CONTRACT_ADDRESS, {
                from: this.clientAccount,
                handleRevert: true,
            },
        );

        // If required to deploy deploys the contract.
        if (shouldDeploy) {
            console.log('Deploying NearBridge smart contract');
            this.nearClientContract = await this.nearClientContract.deploy({
                data: '0x' + fs.readFileSync(path.join(this.nearClientContractPath, 'NearBridge.full.bin')),
            }).send({
                from: this.clientAccount,
                gas: 3000000,
                handleRevert: true,
            });
            console.log('Deployed to address:', this.nearClientContract.address);
        }
    }

    async run () {
        const checkNearStatus = async function () {
            const latest_submitted_block = Number(await this.nearClientContract.methods.lastBlockNumber().call());
            console.log('latest_submitted_block', typeof latest_submitted_block, latest_submitted_block);

            const status = await this.near.connection.provider.status();
            const lastNearBlock = status.sync_info.latest_block_height;
            console.log('lastNearBlock', typeof lastNearBlock, lastNearBlock);

            const promises = [];
            for (let i = latest_submitted_block; i < lastNearBlock; i += 10) {
                promises.push(this.near.connection.provider.block({ blockId: i }));
            }

            const blocks = (await Promise.all(promises)).map(block => {
                return [
                    '0x',
                    this.web3.utils.padLeft(block.header.height.toString(16), 16).match(/../g).reverse().join(''),
                    this.web3.utils.padLeft(this.web3.utils.toHex(bs58.decode(block.header.epoch_id)).substr(2), 64),
                    this.web3.utils.padLeft(this.web3.utils.toHex(bs58.decode(block.header.next_epoch_id)).substr(2), 64),
                    this.web3.utils.padLeft(this.web3.utils.toHex(bs58.decode(block.header.prev_state_root)).substr(2), 64),
                    this.web3.utils.padLeft(this.web3.utils.toHex(bs58.decode(block.header.outcome_root)).substr(2), 64),
                    this.web3.utils.padLeft(block.header.timestamp.toString(16), 16).match(/../g).reverse().join(''),
                    this.web3.utils.padLeft(this.web3.utils.toHex(bs58.decode(block.header.next_bp_hash)).substr(2), 64),
                ].join('');
            });

            // TODO: Investigate how to use new feature web3.eth.handleRevert
            try {
                console.log(`Submitting ${blocks.length} blocks`);
                await this.nearClientContract.methods.addBlockHeaders(blocks).send({
                    gas: 5000000,
                });
                console.log('Sumbitted!');
            } catch (txRevertMessage) {
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

            console.log('Sleep for 10 seconds');
            setTimeout(checkNearStatus, 10000);
        };

        await checkNearStatus();
    }
}

exports.NearRelay = NearRelay;
