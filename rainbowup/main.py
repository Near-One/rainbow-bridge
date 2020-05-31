import argparse
import json
import subprocess
import sys
import os
import time
import urllib
import urllib.parse

from rainbowuplib.ganache_service import GanacheService
from rainbowuplib.ethrelay_service import EthRelayService
from rainbowuplib.ethprover_tester_service import EthProverTesterService

# Port for the local Near node
NEAR_LOCAL_NODE_RPC_PORT = 3030


class RainbowupArgParser(object):

    def __init__(self):
        parser = argparse.ArgumentParser(description='Rainbowup',
                                         usage='''rainbowup <command> [<args>]

Commands are:
    prepare     Test and compile contracts, install dependencies for the relayers and save them in .rainbowup folder.
    run         Start the bridge, including ganache and local near network.
    cleanup     Remove any local files created by the rainbowup, including `.near`, `.nearup` and `.rainbowup`.
    test        Run all tests on the local node and local Ganache.

Run rainbowup <command> --help to see help for specific command.
''')
        parser.add_argument(
            'command',
            help='Subcommand to run',
            choices=['prepare', 'run', 'stop', 'cleanup', 'test'])
        parser.add_argument(
            '--home',
            help=
            "If specified, uses this directory instead of ~ to for .rainbowup files.",
            default=os.path.expanduser(f'~/.rainbowup'))
        parser.add_argument(
            '--source',
            help=
            "If specified, will use Rainbow bridge source in that folder. Otherwise "
            "will use `~/.rainbowup/bridge`. If source does not exist it will "
            "download it from github.")
        parser.add_argument(
            '--nearcore_source',
            help=
            "If specified, will use nearcore source in that folder. Otherwise "
            "will use `~/.rainbowup/core`. If source does not exist it will "
            "download it from github.")
        parser.add_argument(
            '--eth_node_url',
            help=
            'If specified will use external Ethereum node instead of starting'
            'Ganache')
        parser.add_argument(
            '--near_node_url',
            help=
            'If specified, will not start local Near node and will connect to '
            'the specified node. Requires --near_master_key_path to be specified.'
        )
        parser.add_argument(
            '--near_network_id',
            help='If specified, will use this network id instead of `local`.',
            default='local')
        parser.add_argument(
            '--near_master_key_path',
            help='If specified, will use this key and the corresponding '
            'account id to create accounts needed for the bridge.')
        parser.add_argument(
            '--rainbow_environment_image',
            help='If specified, will use this docker image to start EthRelayer'
            ' or NearRelayer instead of native nodejs')
        self.args = parser.parse_args()
        self.args.home = os.path.abspath(self.args.home)
        os.makedirs(self.args.home, exist_ok=True)

        if not self.args.source:
            self.args.source = os.path.join(self.args.home, "source")
        if not os.path.exists(self.args.source):
            subprocess.check_output([
                'git', 'clone', 'https://github.com/near/rainbow-bridge/',
                self.args.source
            ])
            # subprocess.check_output(['git', 'checkout', 'rainbowbridgeup'], cwd=self.args.source)
            print('Downloaded source of the Rainbow Bridge into %s' %
                  self.args.source)
            subprocess.check_output(
                ['git', 'submodule', 'update', '--init', '--recursive'],
                cwd=self.args.source)
            print('Downloaded source submodules')

        if not os.path.exists(os.path.expanduser('~/.nearup')):
            subprocess.check_output([
                'git', 'clone', 'https://github.com/near/nearup/',
                self._nearup_source()
            ])
            print('Downloaded nearup')

        if not self.args.nearcore_source:
            self.args.nearcore_source = os.path.join(self.args.home, "core")
        if not os.path.exists(self.args.nearcore_source):
            subprocess.check_output([
                'git', 'clone', 'https://github.com/nearprotocol/nearcore',
                self.args.nearcore_source
            ])
            print('Downloaded source of the nearcore into %s' %
                  self.args.nearcore_source)

        if self.args.near_master_key_path:
            self.args.near_master_key_path = os.path.abspath(
                self.args.near_master_key_path)

        self.args.home = os.path.abspath(self.args.home)
        getattr(self, self.args.command)()

    def _is_external_node(self):
        return bool(self.args.near_node_url)

    def _near_node_url(self):
        return self.args.near_node_url or 'http://localhost:%s' % NEAR_LOCAL_NODE_RPC_PORT

    def _nearup_source(self):
        return os.path.expanduser('~/.nearup')

    # Try connecting to the Near node.
    def _is_near_node_running(self):
        url = urllib.parse.urlparse(self._near_node_url())
        p = subprocess.Popen(
            ['nc', '-z', url.hostname, str(url.port)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE)
        p.communicate()
        return p.returncode == 0

    # Wait for the operation to return true.
    def _wait(self, f, max_time=60, delta=5):
        for i in range(0, max_time + delta, delta):
            if f():
                return
            else:
                if i != max_time:
                    time.sleep(delta)
                    print("Retrying in %s seconds" % delta)
                else:
                    print("Failed to wait after %s seconds" % max_time)
                    exit(1)

    def _near_datafolder(self):
        return os.path.join(self.args.home, 'near')

    def _eth_node_url(self):
        return self.args.eth_node_url or GanacheService.url()

    # Account id in Near blockchain that can be used by the bridge.
    def _near_master_account_id(self):
        # If external node then read account id from the provided key file
        if self._is_external_node():
            with open(self.args.near_master_key_path, 'r') as f:
                return json.load(f)['account_id']
        else:
            return 'node0'

    def _near_master_sk(self):
        if self._is_external_node():
            with open(self.args.near_master_key_path, 'r') as f:
                return json.load(f)['secret_key']
        else:
            with open(
                    os.path.join(self._near_datafolder(),
                                 'node0/validator_key.json'), 'r') as f:
                return json.load(f)['secret_key']

    def _read_config(self):
        fp = os.path.join(self.args.home, 'config.json')
        if os.path.exists(fp):
            with open(fp, 'r') as f:
                return json.load(f)
        return dict()

    def _write_config(self, config):
        fp = os.path.join(self.args.home, 'config.json')
        with open(fp, 'w+') as f:
            return json.dump(config, f)

    # Read value from .rainbowup/config.json
    def _read_config_kv(self, key):
        return self._read_config().get(key)

    # Write key value to .rainbowup/config.json
    def _write_config_kv(self, key, value):
        config = self._read_config()
        config[key] = value
        self._write_config(config)

    # Remove key value from .rainbowup/config.json
    def _remove_config_kv(self, key):
        config = self._read_config()
        config.pop(key)
        self._write_config(config)

    def prepare(self):
        env=dict(BRIDGE_SRC=self.args.source, CORE_SRC=self.args.nearcore_source)
        env = {**os.environ, **env}
        p = subprocess.Popen(
            ['bash', 'prepare.sh'],
            cwd=os.path.join(self.args.source, 'rainbowup/scripts'),
            env=env
        )
        p.wait()
        if p.returncode:
            exit(1)

    def _run(self):
        # If external node is not specified then we must start local node.
        if not self._is_external_node() and not self._is_near_node_running():
            p = subprocess.Popen([
                'python3', 'main.py', 'localnet', '--num-nodes', '1', '--home',
                self._near_datafolder(), '--binary-path',
                os.path.join(self.args.nearcore_source, 'target/debug')
            ],
                                 cwd=self._nearup_source(),
                                 stdin=subprocess.PIPE)
            p.communicate()
            if p.returncode != 0:
                print("Failed to start the local node")
                exit(1)
            print("Started local node")

        # Wait until the connection to the node is working.
        self._wait(self._is_near_node_running)

        # If Ethereum network is not specified then we need to start Ganache and wait for it.
        if not self.args.eth_node_url:
            d = GanacheService(self.args)
            d.run()
            # We cannot really check the external Ethereum network like that so we only do it for Ganache.
            self._wait(GanacheService.is_running)

        # Start EthRelay daemon.
        d = EthRelayService(
            self.args,
            eth_node_url=self._eth_node_url(),
            near_node_url=self._near_node_url(),
            master_acc_id=self._near_master_account_id(),
            master_sk=self._near_master_sk(),
            bridge_acc_id='ethbridge',
            bridge_sk=self._near_master_sk(),  # Use the same key for now.
            validate_ethash='true' if self.args.eth_node_url else 'false')
        d.run()

    def run(self):
        self._run()
        input("Press Enter to terminate the relay...")

    def cleanup(self):
        if not self._is_external_node() and self._is_near_node_running():
            subprocess.check_output(['python3', 'main.py', 'stop'],
                                    cwd=self._nearup_source())
        # Remove the data and potentially source files.
        subprocess.check_output(['rm', '-rf', self.args.home])

    def test(self):
        # Start EthRelay daemon.
        d = EthProverTesterService(
            self.args,
            eth_node_url=self._eth_node_url(),
            near_node_url=self._near_node_url(),
            master_acc_id=self._near_master_account_id(),
            master_sk=self._near_master_sk(),
            bridge_acc_id='ethbridge',
            bridge_sk=self._near_master_sk(),  # Use the same key for now.
            validate_ethash='true' if self.args.eth_node_url else 'false')
        d.run().wait()

    def _test(self):
        # Run tests on the eth bridge contract
        # subprocess.check_output(['./test.sh'], cwd=os.path.join(self.args.source, 'ethbridge'))
        # Start up the bridge
        self._run()

        # Deploy emitter to Ganache.
        stdout = subprocess.check_output([
            'yarn', 'run', 'oz', 'deploy', 'Emitter', '--kind', 'regular',
            '--network', 'development', '--silent', '--no-interactive'
        ],
                                         cwd=os.path.join(
                                             self.args.source,
                                             'libs-sol/emitter'))
        contract_address = next((s for s in stdout.decode("utf-8").split("\n")
                                 if s.startswith("0x")), None)
        p = subprocess.Popen([
            'yarn', 'run', 'oz', 'send-tx', '--to', contract_address,
            '--method', '"emitEvent(uint256,uint256,uint256)"', '--args',
            '1,2,3', '--network', 'development', '--no-interactive'
        ],
                             cwd=os.path.join(self.args.source,
                                              'libs-sol/emitter'),
                             stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE)
        p.wait()

        stderr = p.stderr.read().decode().strip()
        prefix = '- Transaction successful. Transaction hash: '
        tx_hash = next((s for s in stderr.split("\n") if s.startswith(prefix)),
                       None)[len(prefix):]
        print(stderr)

        env = dict(ETH_NODE_URL=self._eth_node_url(), TX_HASH=tx_hash)
        print(env)
        env = {**os.environ, **env}

        p = subprocess.Popen(['node', 'index.js', 'extract_proof'],
                             env=env,
                             cwd=os.path.join(self.args.source, 'environment'))
        p.wait()


if __name__ == '__main__':
    sys.argv[0] = 'rainbowup'
    arg_parser = RainbowupArgParser()
