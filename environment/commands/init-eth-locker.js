const Web3 = require('web3');
const BN = require('bn.js');
const fs = require('fs');
const { RainbowConfig } = require('../lib/config');
const { normalizeEthKey } = require('../lib/robust');

class InitETHLocker {
    static async execute () {
        // @ts-ignore
        const web3 = new Web3(RainbowConfig.getParam('eth-node-url'));
        let ethMasterAccount =
        web3.eth.accounts.privateKeyToAccount(normalizeEthKey(RainbowConfig.getParam('eth-master-sk')));
        web3.eth.accounts.wallet.add(ethMasterAccount);
        web3.eth.defaultAccount = ethMasterAccount.address;
        ethMasterAccount = ethMasterAccount.address;

        // Initialize Token Locker contract.
        console.log('Deploying Locker contract.');
        const tokenLockerContract = new web3.eth.Contract(JSON.parse(
            // @ts-ignore
            fs.readFileSync(RainbowConfig.getParam('eth-locker-abi-path'))));
        const txContractLocker =
        await tokenLockerContract
            .deploy({
                data: '0x' + fs.readFileSync(RainbowConfig.getParam('eth-locker-bin-path')),
                arguments: [
                    RainbowConfig.getParam('eth-erc20-address'),
                    Buffer.from(RainbowConfig.getParam('near-fun-token-account'), 'utf8'),
                    RainbowConfig.getParam('near2eth-prover-address'),
                ],
            })
            .send({
                from: ethMasterAccount,
                gas: 3000000,
                gasPrice: new BN(await web3.eth.getGasPrice()).mul(new BN(RainbowConfig.getParam('eth-gas-multiplier'))),
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
