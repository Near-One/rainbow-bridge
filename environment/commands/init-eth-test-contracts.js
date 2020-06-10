const Web3 = require('web3');
const fs = require('fs');
const path = require('path');

class InitETHTestContracts {
    static async execute (command) {
        const web3 = new Web3(command.ethNodeUrl);
        let ethMasterAccount =
        web3.eth.accounts.privateKeyToAccount(command.ethMasterSk);
        web3.eth.accounts.wallet.add(ethMasterAccount);
        web3.eth.defaultAccount = ethMasterAccount.address;
        ethMasterAccount = ethMasterAccount.address;
        // let accs = await web3.eth.getAccounts();
        // ethMasterAccount = (await web3.eth.getAccounts())[0];

        // Initialize MyERC20 contract.
        console.log('Deploying MyERC20 contract.');
        const myERC20Contract = new web3.eth.Contract(JSON.parse(
            fs.readFileSync(path.join(command.contractsDir, 'MyERC20.full.abi'))));
        const txERC20 =
        await myERC20Contract
            .deploy({
                data: '0x' + fs.readFileSync(path.join(command.contractsDir,
                    'MyERC20.full.bin')),
            })
            .send({
                from: ethMasterAccount,
                gas: 3000000,
            });
        console.log('Deployed MyERC20 contract to:');
        console.log(`${txERC20.options.address}`);

        // Initialize Token Locker contract.
        console.log('Deploying TokenLocker contract.');
        const tokenLockerContract = new web3.eth.Contract(JSON.parse(fs.readFileSync(
            path.join(command.contractsDir, 'TokenLocker.full.abi'))));
        const txContractLocker =
        await tokenLockerContract
            .deploy({
                data: '0x' + fs.readFileSync(path.join(command.contractsDir,
                    'TokenLocker.full.bin')),
            })
            .send({
                from: ethMasterAccount,
                gas: 3000000,
            });
        console.log('Deployed TokenLocker contract to:');
        console.log(`${txContractLocker.options.address}`);

        try {
            // Only WebSocket provider can close.
            web3.currentProvider.connection.close();
        } catch (e) {
        }
    }
}

exports.InitETHTestContracts = InitETHTestContracts;
