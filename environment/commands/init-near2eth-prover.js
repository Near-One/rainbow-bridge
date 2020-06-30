const Web3 = require('web3');
const fs = require('fs');
const { RainbowConfig } = require('../lib/config');

class InitNear2EthProver {
    static async execute () {
        // @ts-ignore
        const web3 = new Web3(RainbowConfig.getParam('eth-node-url'));
        let ethMasterAccount =
            web3.eth.accounts.privateKeyToAccount(RainbowConfig.getParam('eth-master-sk'));
        web3.eth.accounts.wallet.add(ethMasterAccount);
        web3.eth.defaultAccount = ethMasterAccount.address;
        ethMasterAccount = ethMasterAccount.address;

        // Initialize client contract.
        console.log('Deploying Near2EthClient contract.');
        const nearProver = new web3.eth.Contract(JSON.parse(
            // @ts-ignore
            fs.readFileSync(RainbowConfig.getParam('near2eth-prover-abi-path'))));
        const tx =
            await nearProver
                .deploy({
                    data: '0x' + fs.readFileSync(RainbowConfig.getParam('near2eth-prover-bin-path')),
                    arguments: [RainbowConfig.getParam('near2eth-client-address')],
                })
                .send({
                    from: ethMasterAccount,
                    gas: 3000000,
                });
        console.log('Deployed Near2EthProver contract to:');
        console.log(`${tx.options.address}`);
        RainbowConfig.setParam('near2eth-prover-address', tx.options.address);
        RainbowConfig.saveConfig();

        try {
            // Only WebSocket provider can close.
            web3.currentProvider.connection.close();
        } catch (e) {
        }
    }
}

exports.InitNear2EthProver = InitNear2EthProver;
