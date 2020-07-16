const ProcessManager = require('pm2');
const { spawnProcess } = require('./helpers');
const { Near2EthRelay } = require('../../lib/near2eth-relay');
const { RainbowConfig } = require('../../lib/config');

class StartNearRelayCommand {
    static async execute() {
        if (RainbowConfig.getParam('daemon') === 'true') {
            ProcessManager.connect((err) => {
                if (err) {
                    console.log(
                        'Unable to connect to the ProcessManager daemon! Please retry.');
                    return;
                }
                spawnProcess('near-relay',
                    {
                        name: 'near-relay',
                        script: 'index.js',
                        interpreter: 'node',
                        error_file: '~/.rainbow/logs/near-relay/err.log',
                        out_file: '~/.rainbow/logs/near-relay/out.log',
                        args: ['start', 'near-relay', ...RainbowConfig.getArgsNoDaemon()],
                        wait_ready: true,
                        kill_timeout: 60000,
                        logDateFormat: 'YYYY-MM-DD HH:mm:ss.SSS'

                    },
                );
            });
        } else {
            const relay = new Near2EthRelay();
            await relay.initialize();
            await relay.run();
        }
    }
}

exports.StartNearRelayCommand = StartNearRelayCommand;
