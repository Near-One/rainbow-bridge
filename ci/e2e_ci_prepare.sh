# This file should be sourced when run e2e test on ci
source ~/.nvm/nvm.sh
source ~/.cargo/env
source ~/.yarn/yarn.sh

if [[ ! -d ~/go ]]; then
    wget -q -O - https://raw.githubusercontent.com/canha/golang-tools-install-script/master/goinstall.sh | bash
fi

export GOROOT=~/.go
export GOPATH=~/go
export PATH=$$GOPATH/bin:$$GOROOT/bin:$$PATH

# hard link pm2 logs to current dir, so buildkite can pick up them as artifacts
mkdir -p ~/.rainbow/logs/eth-relay
mkdir -p ~/.rainbow/logs/near-relay
mkdir -p ~/.rainbow/logs/ganache
touch eth-relay-out.log
touch eth-relay-err.log
touch near-relay-out.log
touch near-relay-err.log
touch ganache-out.log
touch ganache-err.log
if [[ ! -f ~/.rainbow/logs/eth-relay/out.log ]]; then
    ln eth-relay-out.log ~/.rainbow/logs/eth-relay/out.log
    ln eth-relay-err.log ~/.rainbow/logs/eth-relay/err.log
    ln near-relay-out.log ~/.rainbow/logs/near-relay/out.log
    ln near-relay-err.log ~/.rainbow/logs/near-relay/err.log
    ln ganache-out.log ~/.rainbow/logs/ganache/out.log
    ln ganache-err.log ~/.rainbow/logs/ganache/err.log
fi