const ProcessManager = require('pm2');
const nearlib = require('nearlib');
const { spawnProcess } = require('./helpers');
const { Eth2NearRelay } = require('../../lib/eth2near-relay');
const { Eth2NearClientContract } = require('../../lib/eth2near-client-contract');

class StartEthRelayCommand {
    // Get args without daemon set on.
    static getNoDaemonArgs (command) {
        return [
            'start',
            'eth-relay',
            '--master-account',
            command.masterAccount,
            '--master-sk',
            command.masterSk,
            '--client-account',
            command.clientAccount,
            '--near-network-id',
            command.nearNetworkId,
            '--near-node-url',
            command.nearNodeUrl,
            '--eth-node-url',
            command.ethNodeUrl,
            '--daemon',
            'false',
        ];
    }

    static async execute (command) {
        if (command.daemon === 'true') {
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
                        args: StartEthRelayCommand.getNoDaemonArgs(command),
                    },
                );
            });
        } else {
            const masterAccount = command.masterAccount;
            const masterSk = command.masterSk;
            const keyStore = new nearlib.keyStores.InMemoryKeyStore();
            await keyStore.setKey(command.nearNetworkId, masterAccount, nearlib.KeyPair.fromString(masterSk));
            const near = await nearlib.connect({
                nodeUrl: command.nearNodeUrl,
                networkId: command.nearNetworkId,
                masterAccount: masterAccount,
                deps: {
                    keyStore: keyStore,
                },
            });

            const relay = new Eth2NearRelay();
            const clientContract =
                new Eth2NearClientContract(new nearlib.Account(near.connection, masterAccount), command.clientAccount);
            await clientContract.accessKeyInit();
            console.log('Initializing Eth-Relay...');
            relay.initialize(clientContract, command.ethNodeUrl);
            console.log('Starting Eth-Relay...');
            await relay.run();
        }
    }
}

exports.StartEthRelayCommand = StartEthRelayCommand;
