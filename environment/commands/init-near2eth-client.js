const Web3 = require('web3');
const fs = require('fs');
const { RainbowConfig } = require('../lib/config');

class InitNear2EthClient {
    static async execute () {
        const web3 = new Web3(RainbowConfig.getParam('eth-node-url'));
        let ethMasterAccount =
            web3.eth.accounts.privateKeyToAccount(RainbowConfig.getParam('eth-master-sk'));
        web3.eth.accounts.wallet.add(ethMasterAccount);
        web3.eth.defaultAccount = ethMasterAccount.address;
        ethMasterAccount = ethMasterAccount.address;

        // Initialize ED25519 contract.

        // Initialize client contract.
        console.log('Deploying Near2EthClient contract.');
        const nearBridge = new web3.eth.Contract(JSON.parse(
            fs.readFileSync(RainbowConfig.getParam('near2eth-client-abi-path'))));
        const tx =
            await nearBridge
                .deploy({
                    data: '0x' + fs.readFileSync(RainbowConfig.getParam('near2eth-client-bin-path')),
                    arguments: [RainbowConfig.getParam('eth-ed25519-address')],
                })
                .send({
                    from: ethMasterAccount,
                    gas: 3000000,
                });
        console.log('Deployed Near2EthClient contract to:');
        console.log(`${tx.options.address}`);
        RainbowConfig.setParam('near2eth-client-address', tx.options.address);
        RainbowConfig.saveConfig();

        try {
            // Only WebSocket provider can close.
            web3.currentProvider.connection.close();
        } catch (e) {
        }
    }
}

exports.InitNear2EthClient = InitNear2EthClient;
