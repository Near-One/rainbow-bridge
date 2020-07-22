const Web3 = require('web3');
const BN = require('bn.js');
const fs = require('fs');
const { RainbowConfig } = require('../lib/config');
const { normalizeEthKey } = require('../lib/robust');

class InitETHERC20 {
    static async execute () {
        // @ts-ignore
        const web3 = new Web3(RainbowConfig.getParam('eth-node-url'));
        let ethMasterAccount =
        web3.eth.accounts.privateKeyToAccount(normalizeEthKey(RainbowConfig.getParam('eth-master-sk')));
        web3.eth.accounts.wallet.add(ethMasterAccount);
        web3.eth.defaultAccount = ethMasterAccount.address;
        ethMasterAccount = ethMasterAccount.address;

        // Initialize an ERC20 contract.
        console.log('Deploying ERC20 contract.');
        const myERC20Contract = new web3.eth.Contract(JSON.parse(
            // @ts-ignore
            fs.readFileSync(RainbowConfig.getParam('eth-erc20-abi-path'))));
        const txERC20 =
        await myERC20Contract
            .deploy({
                data: '0x' + fs.readFileSync(RainbowConfig.getParam('eth-erc20-bin-path')),
            })
            .send({
                from: ethMasterAccount,
                gas: 3000000,
                gasPrice: new BN(await web3.eth.getGasPrice()).mul(new BN(RainbowConfig.getParam('eth-gas-multiplier'))),
            });
        console.log('Deployed ERC20 contract to:');
        console.log(`${txERC20.options.address}`);
        RainbowConfig.setParam('eth-erc20-address', txERC20.options.address);
        RainbowConfig.saveConfig();

        try {
            // Only WebSocket provider can close.
            web3.currentProvider.connection.close();
        } catch (e) {
        }
    }
}

exports.InitETHERC20 = InitETHERC20;
