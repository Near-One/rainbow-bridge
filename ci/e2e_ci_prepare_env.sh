# This file should be sourced when run e2e test on ci
set -ex

pip3 install --user -r requirements.txt
export PATH=~/.local/bin:$PATH
nearup version

source ~/.nvm/nvm.sh
source ~/.cargo/env
source ~/.yarn/yarn.sh

if [[ ! -d ~/go ]]; then
    wget -q -O - https://raw.githubusercontent.com/canha/golang-tools-install-script/master/goinstall.sh | bash
fi

export GOROOT=~/.go
export GOPATH=~/go
export PATH=$GOPATH/bin:$GOROOT/bin:$PATH

node --version
