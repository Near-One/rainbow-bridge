import argparse
import json
import subprocess
import sys
import os
import time
import urllib

# Port for the local Near node
NEAR_LOCAL_NODE_RPC_PORT = 3030
# Port for the local Ganache instance
GANACHE_PORT = 9545
# Key that we use for Ganache PID in the config file.
GANACHE_PID_KEY = 'ganache_pid'

class RainbowupArgParser(object):

    def __init__(self):
        parser = argparse.ArgumentParser(
            description='Rainbowup',
            usage='''rainbowup <command> [<args>]

Commands are:
    prepare     Test and compile contracts, install dependencies for the relayers and save them in .rainbowup folder.
    run         Start the bridge, including ganache and local near network.
    stop        Stop any local services created by the rainbowup: relayers, local Near node, local Ganache instance.
    cleanup     Remove any local files created by the rainbowup, including `.near`, `.nearup` and `.rainbowup`.
    test        Run all tests on the local node and local Ganache.

Run rainbowup <command> --help to see help for specific command.
''')
        parser.add_argument('command', help='Subcommand to run', choices=['prepare', 'run', 'stop', 'cleanup', 'test'])
        parser.add_argument('--home', help="If specified, uses this directory instead of ~ to for .rainbowup files.",
                            default=os.path.expanduser(f'~/.rainbowup'))
        parser.add_argument('--source', help="If specified, will use Rainbow bridge source in that folder. Otherwise "
                                             "will use `~/.rainbowup/source`. If source does not exist it will "
                                             "download it from github.")
        parser.add_argument('--nearup_source', help="If specified, will use Nearup source in that folder. Otherwise "
                                                    "will use `~/.rainbowup/nearup`. If source does not exist it will "
                                                    "download it from github.")
        parser.add_argument('--eth_network', help='If specified will use this Ethereum network, instead of starting '
                                                  'Ganache', choices=['ropstein', 'mainnet'])
        parser.add_argument('--near_node_url', help='If specified, will not start local Near node and will connect to '
                                                    'the specified node. Requires --near_pk_path to be specified.')
        parser.add_argument('--near_pk_path', help='If specified, will use this public key and the corresponding '
                                                   'account id to create accounts needed for the bridge.')
        self.args = parser.parse_args()
        self.args.home = os.path.abspath(self.args.home)
        os.makedirs(self.args.home, exist_ok=True)

        if not self.args.source:
            self.args.source = os.path.join(self.args.home, "source")
        if not os.path.exists(self.args.source):
            subprocess.check_output(['git', 'clone', 'https://github.com/nearprotocol/near-bridge/', self.args.source])
            print('Downloaded source of the Rainbow Bridge into %s' % self.args.source)
            subprocess.check_output(['git', 'submodule', 'update', '--init', '--recursive'], cwd=self.args.source)
            print('Downloaded source submodules')

        if not self.args.nearup_source:
            self.args.nearup_source = os.path.join(self.args.home, "nearup")
        if not os.path.exists(self.args.nearup_source):
            subprocess.check_output(['git', 'clone', 'https://github.com/near/nearup', self.args.nearup_source])
            print('Downloaded source of the Nearup into %s' % self.args.nearup_source)

        if self.args.near_pk_path:
            self.args.near_pk_path = os.path.abspath(self.args.near_pk_path)

        self.args.home = os.path.abspath(self.args.home)
        getattr(self, self.args.command)()

    def _is_external_node(self):
        return bool(self.args.near_node_url)

    def _node_url(self):
        return self.args.near_node_url or 'http://localhost:%s' % NEAR_LOCAL_NODE_RPC_PORT

    # Try connecting to the Near node.
    def _is_near_node_running(self):
        url = urllib.parse.urlparse(self._node_url())
        p = subprocess.Popen(['nc', '-z', url.netloc, str(url.port)], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        p.communicate()
        return p.returncode == 0

    # Try connected to Ganache
    def _is_ganache_running(self):
        p = subprocess.Popen(['nc', '-z', 'localhost', str(GANACHE_PORT)], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        p.communicate()
        return p.returncode == 0

    # Wait for the operation to return true.
    def _wait(self, f, max_time = 60, delta = 5):
        for i in range(0, max_time + delta, delta):
            if f:
                return
            else:
                if i != max_time:
                    time.sleep(delta)
                    print("Cannot reach the node %s. Retrying in %s seconds", self._node_url(), delta)
                else:
                    print("Cannot reach the node %s after %s seconds", self._node_url(), max_time)
                    exit(1)

    # Account id in Near blockchain that can be used by the bridge.
    def _near_account_id(self):
        # If external node then read account id from the provided key file
        if self._is_external_node():
            return json.load(self.args.near_pk_path)['account_id']
        else:
            return 'ethbridge'

    # Read value from .rainbowup/config.json
    def _read_config_kv(self, key):
        fp = os.path.join(self.args.home, 'config.json')
        if os.path.exists(fp):
            return json.load(fp)[key]

    # Write key value to .rainbowup/config.json
    def _write_config_kv(self, key, value):
        fp = os.path.join(self.args.home, 'config.json')
        config = dict()
        if os.path.exists(fp):
             config = json.load(fp)
        if value is None:
            config.pop(key)
        else:
            config[key] = value
        json.dump(fp, config)

    def _remove_config_kv(self, key):
        fp = os.path.join(self.args.home, 'config.json')
        config = dict()
        if os.path.exists(fp):
            config = json.load(fp)
        config.pop(key)
        json.dump(fp, config)

    def prepare(self):
        # Compile Eth Bridge contract
        subprocess.check_output(['./build.sh'], cwd=os.path.join(self.args.source, 'ethbridge'))
        print('Compiled Eth Bridge contract')

        # Copy compiled contract to the home directory
        subprocess.check_output(['cp', os.path.join(self.args.source, 'ethbridge/res/eth_bridge.wasm'), self.args.home])

        # Install EthRelay dependencies
        subprocess.check_output(['yarn'], cwd=os.path.join(self.args.source, 'ethrelay'))
        # Build ethashproof module
        subprocess.check_output(['./build.sh'], cwd=os.path.join(self.args.source, 'ethrelay/ethashproof'))

    def run(self):
        # If external node is not specified then we must start local node.
        if not self._is_external_node():
            p = subprocess.Popen(['main.py', 'devnet'], cwd=self.args.nearup_source, stdin=subprocess.PIPE)
            p.communicate(input=self._near_account_id().encode())
            if p.returncode != 0:
                print("Failed to start the local node")
                exit(1)
            print("Started local node")

        # Wait until the connection to the node is working.
        self._wait(self._is_near_node_running)

        # If Ethereum network is not specified then we need to start Ganache and wait for it.
        if not self.arg.eth_network:
            accounts = [
                '--account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200,1000000000000000000000000"',
                '--account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501201,1000000000000000000000000"',
                '--account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501202,1000000000000000000000000"',
                '--account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501203,1000000000000000000000000"',
                '--account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501204,1000000000000000000000000"',
                '--account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501205,1000000000000000000000000"',
                '--account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501206,1000000000000000000000000"',
                '--account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501207,1000000000000000000000000"',
                '--account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501208,1000000000000000000000000"',
                '--account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501209,1000000000000000000000000"'
            ]
            p = subprocess.Popen(['yarn', 'run', 'ganache-cli', '--blockTime', '12', '--gasLimit', '10000000', '-p', str(GANACHE_PORT)] + accounts, cwd=os.path.join(self.args.source, 'ethrelay'))
            p.communicate()
            self._write_config_kv(GANACHE_PID_KEY, str(p.pid))
            # We cannot really check the external Ethereum network like that so we only do it for Ganache.
            self._wait(self._is_ganache_running())

    def stop(self):
        # If external node is not specified then it must have been run locally.
        if not self._is_external_node():
            subprocess.check_output(['main.py', 'stop'], cwd=self.args.nearup_source)
        # If local Ganache was started then stop it.
        if self._read_config_kv(GANACHE_PID_KEY):
            subprocess.check_output(['kill', self._remove_config_kv(GANACHE_PID_KEY)])
            self._remove_config_kv(GANACHE_PID_KEY)

    def cleanup(self):
        # Remove the data and potentially source files.
        subprocess.check_output(['rm', '-rf', self.args.home])

    def test(self):
        # Run tests on the eth bridge contract
        subprocess.check_output(['./test.sh'], cwd=os.path.join(self.args.source, 'ethbridge'))


if __name__ == '__main__':
    sys.argv[0] = 'rainbowup'
    arg_parser = RainbowupArgParser()
