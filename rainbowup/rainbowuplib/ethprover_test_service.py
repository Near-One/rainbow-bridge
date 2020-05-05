import os
import subprocess


class EthProverTestService:
    def __init__(self, args,
                 eth_node_url,
                 near_node_url,
                 emitter_contract_address,
                 master_acc_id,
                 bridge_acc_id,
                 prover_acc_id,
                 bridge_sk,
                 master_sk,
                 prover_sk
                 ):
        self.args = args
        self.eth_node_url = eth_node_url
        self.near_node_url = near_node_url
        self.emitter_contract_address = emitter_contract_address
        self.master_acc_id = master_acc_id
        self.bridge_acc_id = bridge_acc_id
        self.prover_acc_id = prover_acc_id
        self.master_sk = master_sk
        self.bridge_sk = bridge_sk
        self.prover_sk = prover_sk

    def run(self):
        env = dict(
            ETHEREUM_NODE_URL=self.eth_node_url,
            EMITTER_CONTRACT_ADDRESS=self.emitter_contract_address,
            NEAR_NODE_URL=self.near_node_url,
            NEAR_NODE_NETWORK_ID=self.args.near_network_id,
            MASTER_ACC_ID=self.master_acc_id,
            BRIDGE_ACC_ID=self.bridge_acc_id,
            PROVER_ACC_ID=self.prover_acc_id,
            MASTER_SK=self.master_sk,
            BRIDGE_SK=self.bridge_sk,
            PROVER_SK=self.prover_sk
        )
        print(env)
        env = {**os.environ, **env}
        subprocess.Popen(['node', 'index.js'], env=env, cwd=os.path.join(self.args.source, 'ethprover'), shell=False)
