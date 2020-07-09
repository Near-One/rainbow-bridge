const Web3 = require('web3');
const fs = require('fs');
const { RainbowConfig } = require('./config');
const { web3GetBlockNumber, web3GetBlock } = require('../lib/robust');

function sleep (ms) {
    return new Promise((resolve) => {
        setTimeout(resolve, ms);
    });
}

class Near2EthWatchdog {
    async initialize () {
        // @ts-ignore
        this.web3 = new Web3(RainbowConfig.getParam('eth-node-url'));
        const ethMasterAccount =
            this.web3.eth.accounts.privateKeyToAccount(RainbowConfig.getParam('eth-master-sk'));
        this.web3.eth.accounts.wallet.add(ethMasterAccount);
        this.web3.eth.defaultAccount = ethMasterAccount.address;
        this.ethMasterAccount = ethMasterAccount.address;

        // Initialize client contract.
        console.log('Deploying Near2EthClient contract.');
        this.clientContract = new this.web3.eth.Contract(
            // @ts-ignore
            JSON.parse(fs.readFileSync(RainbowConfig.getParam('near2eth-client-abi-path'))),
            RainbowConfig.getParam('near2eth-client-address'), {
                from: this.ethMasterAccount,
                handleRevert: true,
            },
        );
    }

    async run () {
        while (true) {
            const lastClientBlock = await this.clientContract.methods.last().call();
            const latestBlock = await web3GetBlock(this.web3, 'latest');
            console.log(`Examining block ${lastClientBlock.hash} height: ${lastClientBlock.height}`);
            if (latestBlock.timestamp >= lastClientBlock.valid) {
                const timeDelta = 10;
                console.log(`Block is valid. Sleeping for ${timeDelta} seconds.`);
                await sleep(timeDelta * 1000);
                continue;
            }

            // We cannot memorize processed blocks because they might have been re-submitted with different data.
            for (let i = 0; i < lastClientBlock.approvals_after_next_length; i++) {
                console.log(`Checking ${i} signature.`);
                const result = await this.clientContract.methods.checkBlockProducerSignatureInLastBlock(i).call();
                if (!result) {
                    console.log(`Challenging ${i} signature.`);
                    try {
                        await this.clientContract.methods.challenge(this.ethMasterAccount, i).send({
                            from: this.ethMasterAccount,
                            gas: 5000000,
                        },
                        );
                    } catch (err) {
                        console.log(`Challenge failed. Maybe the block was already reverted? ${err}`);
                    }
                    break;
                }
            }
            const timeDelta = 10;
            console.log(`Sleeping for ${timeDelta} seconds`);
            await sleep(timeDelta * 1000);
        }
    }
}

exports.Near2EthWatchdog = Near2EthWatchdog;
