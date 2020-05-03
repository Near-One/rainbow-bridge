import os
import subprocess


class EthRelayService:
    def __init__(self, args,
                 eth_node_url,
                 near_node_url,
                 master_acc_id,
                 master_sk,
                 bridge_acc_id,
                 bridge_sk,
                 ):
        self.args = args
        self.eth_node_url = eth_node_url
        self.near_node_url = near_node_url
        self.master_acc_id = master_acc_id
        self.master_sk = master_sk
        self.bridge_acc_id = bridge_acc_id
        self.bridge_sk = bridge_sk

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
            ETH_CLIENT_CONTRACT_PATH=os.path.join(self.args.source, 'ethbridge/res/eth_bridge.wasm'),

            ETH_PROVER_ACC_ID="ethprover",
            ETH_PROVER_SK=self.bridge_sk,
            ETH_PROVER_INIT_BALANCE="1000000000000000000000000000",
            ETH_PROVER_CONTRACT_PATH=os.path.join(self.args.source, 'ethprover/res/eth_prover.wasm')
        )
        print(env)
        env = {**os.environ, **env}
        subprocess.Popen(['node', 'index.js', 'start_ethrelay'], env=env, cwd=os.path.join(self.args.source, 'services'), shell=False)
