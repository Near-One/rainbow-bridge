# This file should be sourced when run e2e test on ci
if [[ ! -d ~/go ]]; then
    wget -q -O - https://raw.githubusercontent.com/canha/golang-tools-install-script/master/goinstall.sh | bash
fi

export GOROOT=~/.go
export GOPATH=~/go
export PATH=$GOPATH/bin:$GOROOT/bin:$PATH

pip3 install nearup --upgrade --user
USER_BASE_BIN=$(python3 -m site --user-base)/bin
export PATH="$USER_BASE_BIN:$PATH"
source ~/.profile
