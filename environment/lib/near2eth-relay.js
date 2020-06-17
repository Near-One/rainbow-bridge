const Web3 = require('web3');
const nearlib = require('nearlib');
const fs = require('fs');
const { RainbowConfig } = require('./config');

class Near2EthRelay {
    async initialize() {
        this.web3 = new Web3(RainbowConfig.getParam('eth-node-url'));
        this.ethMasterAccount =
            this.web3.eth.accounts.privateKeyToAccount(RainbowConfig.getParam('eth-master-sk'));
        this.web3.eth.accounts.wallet.add(this.ethMasterAccount);
        this.web3.eth.defaultAccount = this.ethMasterAccount.address;
        this.ethMasterAccount = this.ethMasterAccount.address;

        const keyStore = new nearlib.keyStores.InMemoryKeyStore();
        this.near = await nearlib.connect({
            nodeUrl: RainbowConfig.getParam('near-node-url'),
            networkId: RainbowConfig.getParam('near-network-id'),
            deps: {
                keyStore: keyStore,
            },
        });

        // Declare Near2EthClient contract.
        this.clientContract = new this.web3.eth.Contract(
            JSON.parse(fs.readFileSync(RainbowConfig.getParam('near2eth-client-abi-path'))),
            RainbowConfig.getParam('near2eth-client-address'), {
                from: this.ethMasterAccount,
                handleRevert: true,
            },
        );

        // Check if initialization is needed.
        try {
            console.log('Checking whether client is initialized.');
            const isInitialized = await this.clientContract.methods.initialized().call();
            if (!isInitialized) {
                // Get most recent block from Near blockchain.
                const status = await this.near.connection.provider.status();
                const latestBlockHash = status.sync_info.latest_block_hash;
                const lightClientBlock = await this.near.connection.provider.sendJsonRpc('next_light_client_block', [latestBlockHash]);
                console.log(`${lightClientBlock}`);
            }
        } catch (txRevertMessage) {
            console.log('Failure.');
            console.log(txRevertMessage.toString());
            process.exit(1);
        }
    }
}

exports.Near2EthRelay = Near2EthRelay;
