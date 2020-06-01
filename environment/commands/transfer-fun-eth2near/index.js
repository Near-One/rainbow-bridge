const Web3 = require('web3');

class TransferFunETH2NEAR {
    static async execute(command) {
        let web3 = new Web3(command.ethNodeUrl);
        let ethSenderAccount = web3.eth.accounts.privateKeyToAccount(command.ethSenderSk);
        web3.eth.accounts.wallet.add(ethSenderAccount);
        web3.eth.defaultAccount = ethSenderAccount.address;
        ethSenderAccount = (await web3.eth.getAccounts())[0];

        let ethTokenLockerContract = new web3.eth.Contract(
            JSON.parse(fs.readFileSync(command.ethLockerAbiPath)),
            command.ethLockerAddress
        );
        try {
            console.log("Transferring tokens from the ERC20 account to the token locker account.");
            let transaction = await ethTokenLockerContract.methods.lockToken(command.ethTokenAddress, command.amount, command.nearReceiverAccount)
                .send({
                    from: ethSenderAccount,
                    gas: 5000000,
                    handleRevert: true,
                });
            console.log("Success.");
        } catch (txRevertMessage) {
            console.log("Failure.");
            console.log(txRevertMessage.toString());
            process.exit(1);
        }

        // TODO: Extract proof and feed to the Near contracts.
    }
}

exports.TransferFunETH2NEAR = TransferFunETH2NEAR;
