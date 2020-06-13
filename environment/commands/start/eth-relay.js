const ProcessManager = require('pm2');
const nearlib = require('nearlib');
const { spawnProcess } = require('./helpers');
const { Eth2NearRelay } = require('../../lib/eth2near-relay');
const { Eth2NearClientContract } = require('../../lib/eth2near-client-contract');
const { RainbowConfig } = require('../../lib/config');

class StartEthRelayCommand {
    static async execute () {
        if (RainbowConfig.getParam('daemon') === 'true') {
            ProcessManager.connect((err) => {
                if (err) {
                    console.log(
                        'Unable to connect to the ProcessManager daemon! Please retry.');
                    return;
                }
                spawnProcess('eth-relay',
                    {
                        name: 'eth-relay',
                        script: 'index.js',
                        interpreter: 'node',
                        error_file: '~/.rainbowup/logs/eth-relay/err.log',
                        out_file: '~/.rainbowup/logs/eth-relay/out.log',
                        args: ['start', 'eth-relay', ...RainbowConfig.getArgsNoDaemon()]
                    }
                );
            });
        } else {
            const masterAccount = RainbowConfig.getParam('near-master-account');
            const masterSk = RainbowConfig.getParam('near-master-sk');
            let keyStore = new nearlib.keyStores.InMemoryKeyStore();
            await keyStore.setKey(RainbowConfig.getParam('near-network-id'), masterAccount, nearlib.KeyPair.fromString(masterSk));
            let near = await nearlib.connect({
                nodeUrl: RainbowConfig.getParam('near-node-url'),
                networkId: RainbowConfig.getParam('near-network-id'),
                masterAccount: masterAccount,
                deps: {
                    keyStore: keyStore,
                },
            });

            const relay = new Eth2NearRelay();
            const clientContract =
                new Eth2NearClientContract(new nearlib.Account(near.connection, masterAccount), RainbowConfig.getParam('eth2near-client-account'));
            await clientContract.accessKeyInit();
            relay.initialize(clientContract, RainbowConfig.getParam('eth-node-url'));
            await relay.run();
        }
    }
}

exports.StartEthRelayCommand = StartEthRelayCommand;
