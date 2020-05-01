import os
import subprocess
from rainbowup.rainbowuplib.daemon import Daemon


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
            ETHEREUM_NODE_URL=self.eth_node_url,
            NEAR_NODE_URL=self.near_node_url,
            NEAR_NODE_NETWORK_ID=self.args.near_network_id,
            MASTER_ACC_ID=self.master_acc_id,
            MASTER_SK=self.master_sk,
            BRIDGE_ACC_ID=self.bridge_acc_id,
            BRIDGE_SK=self.bridge_sk,
            BRIDGE_CONTRACT_PATH=os.path.join(self.args.source, 'ethbridge/res/eth_bridge.wasm')
        )
        print(env)
        env = {**os.environ, **env}
        subprocess.Popen(['node', 'index.js'], env=env, cwd=os.path.join(self.args.source, 'ethrelay'), shell=False)
