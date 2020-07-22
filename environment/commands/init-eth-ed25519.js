const Web3 = require('web3');
const BN = require('bn.js');
const fs = require('fs');
const { RainbowConfig } = require('../lib/config');
const { normalizeEthKey } = require('../lib/robust');

class InitEthEd25519 {
    static async execute () {
        // @ts-ignore
        const web3 = new Web3(RainbowConfig.getParam('eth-node-url'));
        let ethMasterAccount =
            web3.eth.accounts.privateKeyToAccount(normalizeEthKey(RainbowConfig.getParam('eth-master-sk')));
        web3.eth.accounts.wallet.add(ethMasterAccount);
        web3.eth.defaultAccount = ethMasterAccount.address;
        ethMasterAccount = ethMasterAccount.address;

        // Initialize ED25519 contract.
        console.log('Deploying ED25519 contract.');
        const nearBridge = new web3.eth.Contract(JSON.parse(
            // @ts-ignore
            fs.readFileSync(RainbowConfig.getParam('eth-ed25519-abi-path'))));
        const tx =
            await nearBridge
                .deploy({
                    data: '0x' + fs.readFileSync(RainbowConfig.getParam('eth-ed25519-bin-path')),
                })
                .send({
                    from: ethMasterAccount,
                    gas: 5000000,
                    gasPrice: new BN(await web3.eth.getGasPrice()).mul(new BN(RainbowConfig.getParam('eth-gas-multiplier'))),
                });
        console.log('Deployed ED25519 contract to:');
        console.log(`${tx.options.address}`);
        RainbowConfig.setParam('eth-ed25519-address', tx.options.address);
        RainbowConfig.saveConfig();

        try {
            // Only WebSocket provider can close.
            web3.currentProvider.connection.close();
        } catch (e) {
        }
    }
}

exports.InitEthEd25519 = InitEthEd25519;
