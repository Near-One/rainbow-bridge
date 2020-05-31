import os
import subprocess


class EthRelayService:

    def __init__(self, args, eth_node_url, near_node_url, master_acc_id,
                 master_sk, bridge_acc_id, bridge_sk, validate_ethash):
        self.args = args
        self.eth_node_url = eth_node_url
        self.near_node_url = near_node_url
        self.master_acc_id = master_acc_id
        self.master_sk = master_sk
        self.bridge_acc_id = bridge_acc_id
        self.bridge_sk = bridge_sk
        self.validate_ethash = validate_ethash

    def run(self):
        env = dict(ETH_NODE_URL=self.eth_node_url,
                   NEAR_NODE_URL=self.near_node_url,
                   NEAR_NODE_NETWORK_ID=self.args.near_network_id,
                   MASTER_ACC_ID=self.master_acc_id,
                   MASTER_SK=self.master_sk,
                   ETH_CLIENT_ACC_ID=self.bridge_acc_id,
                   ETH_CLIENT_SK=self.bridge_sk,
                   ETH_CLIENT_INIT_BALANCE="1000000000000000000000000000",
                   ETH_CLIENT_CONTRACT_PATH=os.path.join(
                       self.args.source, 'libs-rs/res/eth_client.wasm'),
                   ETH_PROVER_ACC_ID="ethprover",
                   ETH_PROVER_SK=self.bridge_sk,
                   ETH_PROVER_INIT_BALANCE="1000000000000000000000000000",
                   ETH_PROVER_CONTRACT_PATH=os.path.join(
                       self.args.source, 'libs-rs/res/eth_prover.wasm'),
                   VALIDATE_ETHASH=self.validate_ethash,

                   NEAR_TOKEN_ACC_ID="funtoken",
                   NEAR_TOKEN_SK=self.bridge_sk,
                   NEAR_TOKEN_INIT_NEAR_BALANCE="1000000000000000000000000000",
                   NEAR_TOKEN_CONTRACT_PATH=os.path.join(
                       self.args.source, 'libs-rs/res/fungible_token.wasm'),

                   NEAR_LOCKER_ACC_ID="nearlocker",
                   NEAR_LOCKER_SK=self.bridge_sk,
                   NEAR_LOCKER_INIT_NEAR_BALANCE="1000000000000000000000000000",
                   NEAR_LOCKER_INIT_TOKEN_BALANCE="1000000",
                   NEAR_LOCKER_CONTRACT_PATH=os.path.join(
                       self.args.source, 'libs-rs/res/locker.wasm')
                   )
        print(env)
        if self.args.rainbow_environment_image:
            env_list = sum(list(map(lambda k: ['-e', k + '=' + env[k]], env)),
                           [])
            client_contract_path = os.path.abspath(
                env["ETH_CLIENT_CONTRACT_PATH"])
            prover_contract_path = os.path.abspath(
                env["ETH_PROVER_CONTRACT_PATH"])
            subprocess.Popen([
                'docker',
                'run',
                '--network',
                'host',
                '-v',
                f'{client_contract_path}:{client_contract_path}',
                '-v',
                f'{prover_contract_path}:{prover_contract_path}',
                *env_list,
                self.args.rainbow_environment_image,
                'start',
                'start_ethrelay',
            ])
        else:
            env = {**os.environ, **env}
            subprocess.Popen(['node', 'index.js', 'start', 'start_ethrelay'],
                             env=env,
                             cwd=os.path.join(self.args.source, 'environment'),
                             shell=False)
