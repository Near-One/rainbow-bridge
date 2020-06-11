const ProcessManager = require('pm2');
const { spawnProcess } = require('./helpers');
const ganache = require('ganache-core');

const GANACHE_PORT = 9545;

class StartGanacheNodeCommand {
    static async execute (command) {
        if (command.daemon === 'true') {
            ProcessManager.connect((err) => {
                if (err) {
                    console.log(
                        'Unable to connect to the ProcessManager daemon! Please retry.');
                    return;
                }
                spawnProcess('ganache',
                    {
                        name: 'ganache',
                        script: 'index.js',
                        interpreter: 'node',
                        error_file: '~/.rainbowup/logs/ganache/err.log',
                        out_file: '~/.rainbowup/logs/ganache/out.log',
                        args: ['start', 'ganache', '--daemon', 'false']
                    }
                );
            });
        } else {
            const server = ganache.server({
                logger: console,
                accounts: [
                    {
                        secretKey: '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200',
                        balance: 1000000000000000000000000
                    },
                    {
                        secretKey: '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501201',
                        balance: 1000000000000000000000000
                    },
                    {
                        secretKey: '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501202',
                        balance: 1000000000000000000000000
                    },
                    {
                        secretKey: '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501203',
                        balance: 1000000000000000000000000
                    },
                    {
                        secretKey: '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501204',
                        balance: 1000000000000000000000000
                    },
                    {
                        secretKey: '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501205',
                        balance: 1000000000000000000000000
                    },
                    {
                        secretKey: '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501206',
                        balance: 1000000000000000000000000
                    },
                    {
                        secretKey: '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501207',
                        balance: 1000000000000000000000000
                    },
                    {
                        secretKey: '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501208',
                        balance: 1000000000000000000000000
                    },
                    {
                        secretKey: '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501209',
                        balance: 1000000000000000000000000
                    },
                ],
                blockTime: 12,
                gasLimit: 10000000,
                port: GANACHE_PORT
            });
            server.listen(GANACHE_PORT, function (err, blockchain) {
                if (err) {
                    console.log(`Ganache error ${err} on blockchain ${blockchain}`);
                }
            });
        }
    }
}

exports.StartGanacheNodeCommand = StartGanacheNodeCommand;
