import argparse
import subprocess
import sys
import os


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

        self.args.home = os.path.abspath(self.args.home)
        getattr(self, self.args.command)()

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
        pass

    def stop(self):
        # If external node is not specified, that it must have been run locally.
        if not self.args.node_url:
            subprocess.check_output(['main.py', 'stop'], cwd=self.args.nearup_source)

    def cleanup(self):
        # Remove the data and potentially source files.
        subprocess.check_output(['rm', '-rf', self.args.home])

    def test(self):
        # Run tests on the eth bridge contract
        subprocess.check_output(['./test.sh'], cwd=os.path.join(self.args.source, 'ethbridge'))


if __name__ == '__main__':
    sys.argv[0] = 'rainbowup'
    arg_parser = RainbowupArgParser()
