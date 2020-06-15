const Web3 = require('web3');
const fs = require('fs');
const path = require('path');
const { RainbowConfig } = require('../lib/config');

class InitETHLocker {
    static async execute () {
        const web3 = new Web3(RainbowConfig.getParam('eth-node-url'));
        let ethMasterAccount =
        web3.eth.accounts.privateKeyToAccount(RainbowConfig.getParam('eth-master-sk'));
        web3.eth.accounts.wallet.add(ethMasterAccount);
        web3.eth.defaultAccount = ethMasterAccount.address;
        ethMasterAccount = ethMasterAccount.address;

        // Initialize Token Locker contract.
        console.log('Deploying Locker contract.');
        const tokenLockerContract = new web3.eth.Contract(JSON.parse(fs.readFileSync(
            RainbowConfig.getParam('eth-locker-abi-path'))));
        const txContractLocker =
        await tokenLockerContract
            .deploy({
                data: '0x' + fs.readFileSync(RainbowConfig.getParam('eth-locker-bin-path')),
            })
            .send({
                from: ethMasterAccount,
                gas: 3000000,
            });
        console.log('Deployed TokenLocker contract to:');
        console.log(`${txContractLocker.options.address}`);
        RainbowConfig.setParam('eth-locker-address', txContractLocker.options.address);
        RainbowConfig.saveConfig();

        try {
            // Only WebSocket provider can close.
            web3.currentProvider.connection.close();
        } catch (e) {
        }
    }
}

exports.InitETHLocker = InitETHLocker;
