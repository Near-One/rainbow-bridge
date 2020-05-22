import os
import subprocess


class EthProverTesterService:
    def __init__(self, args,
                 eth_node_url,
                 near_node_url,
                 master_acc_id,
                 master_sk,
                 bridge_acc_id,
                 bridge_sk,
                 validate_ethash
                 ):
        self.args = args
        self.eth_node_url = eth_node_url
        self.near_node_url = near_node_url
        self.master_acc_id = master_acc_id
        self.master_sk = master_sk
        self.bridge_acc_id = bridge_acc_id
        self.bridge_sk = bridge_sk
        self.validate_ethash = validate_ethash

    def run(self):
        env = dict(
            ETH_NODE_URL=self.eth_node_url,
            NEAR_NODE_URL=self.near_node_url,
            NEAR_NODE_NETWORK_ID=self.args.near_network_id,
            MASTER_ACC_ID=self.master_acc_id,
            MASTER_SK=self.master_sk,

            ETH_CLIENT_ACC_ID=self.bridge_acc_id,
            ETH_CLIENT_SK=self.bridge_sk,
            ETH_CLIENT_INIT_BALANCE="1000000000000000000000000000",
            ETH_CLIENT_CONTRACT_PATH=os.path.join(self.args.source, 'libs-rs/res/eth_client.wasm'),

            ETH_PROVER_ACC_ID="ethprover",
            ETH_PROVER_SK=self.bridge_sk,
            ETH_PROVER_INIT_BALANCE="1000000000000000000000000000",
            ETH_PROVER_CONTRACT_PATH=os.path.join(self.args.source, 'libs-rs/res/eth_prover.wasm'),
            VALIDATE_ETHASH=self.validate_ethash
        )
        print(env)
        if self.args.rainbow_environment_image:
            env_list = sum(list(map(lambda k: ['-e', k + '=' + env[k]], env)), [])
            client_contract_path = os.path.abspath(env["ETH_CLIENT_CONTRACT_PATH"])
            prover_contract_path = os.path.abspath(env["ETH_PROVER_CONTRACT_PATH"])
            return subprocess.Popen(['docker', 'run', '--network', 'host',
                                     '-v', f'{client_contract_path}:{client_contract_path}',
                                     '-v', f'{prover_contract_path}:{prover_contract_path}',
                                     *env_list, self.args.rainbow_environment_image, 'start_ethrelay'])
        else:
            env = {**os.environ, **env}
            return subprocess.Popen(['node', 'index.js', 'test_ethprover'], env=env, cwd=os.path.join(self.args.source, 'environment'), shell=False)
