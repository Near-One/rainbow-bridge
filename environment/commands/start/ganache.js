const util = require('util');
const ProcessManager = require('pm2');
const { spawnProcess } = require('./helpers');

const GANACHE_PORT = 9545;

class StartGanacheNodeCommand {
    static getGanacheConfig (port = GANACHE_PORT) {
        let accounts = '';
        const accountPrefix =
        'x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b750120';

        for (let i = 0; i < 10; i++) {
            accounts += util.format(
                '--account=%s%d,1000000000000000000000000 ',
                accountPrefix,
                i,
            );
        }

        const command = 'yarn run ganache-cli';
        const args = util.format(
            '--blocktime 12 --gasLimit 10000000 %s -p %d',
            accounts,
            port,
        );

        return {
            name: 'ganache',
            script: command,
            args: args,
            error_file: '~/.rainbowup/logs/ganache/err.log',
            out_file: '~/.rainbowup/logs/ganache/out.log',
        };
    }

    static async execute (command) {
        ProcessManager.connect((err) => {
            if (err) {
                console.log(
                    'Unable to connect to the ProcessManager deamon! Please retry.');
                return;
            }
            spawnProcess('ganache', StartGanacheNodeCommand.getGanacheConfig());
        });
    }
}

exports.StartGanacheNodeCommand = StartGanacheNodeCommand;
