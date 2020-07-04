const ProcessManager = require('pm2');
const { spawnProcess } = require('./helpers');
const { Near2EthWatchdog } = require('../../lib/near2eth-watchdog');
const { RainbowConfig } = require('../../lib/config');

class StartNearWatchdogCommand {
    static async execute () {
        if (RainbowConfig.getParam('daemon') === 'true') {
            ProcessManager.connect((err) => {
                if (err) {
                    console.log(
                        'Unable to connect to the ProcessManager daemon! Please retry.');
                    return;
                }
                spawnProcess('near-watchdog',
                    {
                        name: 'near-watchdog',
                        script: 'index.js',
                        interpreter: 'node',
                        error_file: '~/.rainbowup/logs/near-watchdog/err.log',
                        out_file: '~/.rainbowup/logs/near-watchdog/out.log',
                        args: ['start', 'near-relay', ...RainbowConfig.getArgsNoDaemon()],
                    },
                );
            });
        } else {
            const watchdog = new Near2EthWatchdog();
            await watchdog.initialize();
            await watchdog.run();
        }
    }
}

exports.StartNearWatchdogCommand = StartNearWatchdogCommand;
