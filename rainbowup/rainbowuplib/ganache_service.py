import os
import subprocess
import time

# Port for the local Ganache instance
GANACHE_PORT = 9545


class GanacheService:

    def __init__(self, args):
        self.args = args

    @staticmethod
    def is_running():
        p = subprocess.Popen(['nc', '-z', 'localhost',
                              str(GANACHE_PORT)],
                             stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE)
        p.communicate()
        return p.returncode == 0

    @staticmethod
    def url():
        return 'ws://localhost:%s' % GANACHE_PORT

    def run(self):
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
        subprocess.Popen([
            'yarn', 'run', 'ganache-cli', '--blockTime', '12', '--gasLimit',
            '10000000', '-p',
            str(GANACHE_PORT)
        ] + accounts,
                         cwd=os.path.join(self.args.source, 'environment'),
                         shell=False)
