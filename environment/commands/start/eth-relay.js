const fs = require('fs');
const os = require('os');

const ProcessManager = require('pm2');
const {
    spawnProcess, getLocalGanacheNodeURL, getLocalNearNodeURL,
} = require('./helpers');

class StartEthRelayCommand {
    static getEthRelayConfig (command) {
        let nearNodeUrl = command.nearNodeUrl;
        let nearMasterAccount = command.nearMasterAccount;
        let nearMasterSk = command.nearMasterSk;
        let ethNodeUrl = command.ethNodeUrl;
        let validateEthHash = 'false';

        const homeDir = os.homedir();
        if (command.ethNodeUrl) {
            validateEthHash = 'true';
        }

        if (!command.nearMasterAccount | !command.nearMasterSk) {
            const config = JSON.parse(fs.readFileSync(
                homeDir + '/.near/localnet/node0/validator_key.json', 'utf8'));
            nearMasterAccount = config.account_id;
            nearMasterSk = config.secret_key;
        }

        if (!ethNodeUrl) {
            ethNodeUrl = getLocalGanacheNodeURL();
        }

        if (!nearNodeUrl) {
            nearNodeUrl = getLocalNearNodeURL();
        }

        return {
            name: 'eth-relay',
            script: 'node lib/eth-relay/start.js',
            error_file: '~/.rainbowup/logs/eth-relay/err.log',
            out_file: '~/.rainbowup/logs/eth-relay/out.log',
            env: {
                ETH_NODE_URL: ethNodeUrl,
                NEAR_NODE_URL: nearNodeUrl,
                NEAR_NODE_NETWORK_ID: command.nearNetworkId,
                MASTER_ACC_ID: nearMasterAccount,
                MASTER_SK: nearMasterSk,
                ETH_CLIENT_ACC_ID: 'ethbridge',
                ETH_CLIENT_SK: nearMasterSk,
                ETH_CLIENT_INIT_BALANCE: '100000000000000000000000000',
                ETH_CLIENT_CONTRACT_PATH:
            homeDir + '/.rainbowup/bridge/libs-rs/res/eth_client.wasm',
                ETH_PROVER_ACC_ID: 'ethprover0',
                ETH_PROVER_SK: nearMasterSk,
                ETH_PROVER_INIT_BALANCE: '100000000000000000000000000',
                ETH_PROVER_CONTRACT_PATH:
            homeDir + '/.rainbowup/bridge/libs-rs/res/eth_prover.wasm',
                VALIDATE_ETHASH: validateEthHash,

                NEAR_TOKEN_ACC_ID: 'funtoken0',
                NEAR_TOKEN_SK: nearMasterSk,
                NEAR_TOKEN_INIT_NEAR_BALANCE: '100000000000000000000000000',
                NEAR_TOKEN_CONTRACT_PATH:
            homeDir + '/.rainbowup/bridge/libs-rs/res/fungible_token.wasm',

                NEAR_LOCKER_ACC_ID: 'nearlocker0',
                NEAR_LOCKER_SK: nearMasterSk,
                NEAR_LOCKER_INIT_NEAR_BALANCE: '100000000000000000000000000',
                NEAR_LOCKER_INIT_TOKEN_BALANCE: '1000000',
                NEAR_LOCKER_CONTRACT_PATH:
            homeDir + '/.rainbowup/bridge/libs-rs/res/locker.wasm',
            },
        };
    }

    static async execute (command) {
        ProcessManager.connect((err) => {
            if (err) {
                console.log(
                    'Unable to connect to the ProcessManager deamon! Please retry.');
                return;
            }
            spawnProcess('eth-relay',
                StartEthRelayCommand.getEthRelayConfig(command));
        });
    }
}

exports.StartEthRelayCommand = StartEthRelayCommand;
