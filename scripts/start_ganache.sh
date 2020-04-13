#!/usr/bin/env bash

_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

source $_DIR/waitport.sh
source $_DIR/trapadd.sh

ganache_port=9545

# Exit script as soon as a command fails.
set -o errexit

# Executes cleanup function at script exit.
trap_add cleanup_ganache EXIT

cleanup_ganache() {
    # Kill the ganache instance that we started (if we started one and if it's still running).
    if [ -n "$ganache_pid" ] && ps -p $ganache_pid > /dev/null; then
        kill $ganache_pid
    fi
}

ganache_running() {
    nc -z localhost "$ganache_port"
}

start_ganache() {
    local accounts=(
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501201,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501202,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501203,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501204,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501205,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501206,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501207,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501208,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501209,1000000000000000000000000"
    )

    yarn run ganache-cli --blockTime 12 --gasLimit 10000000 -p "$ganache_port" "${accounts[@]}" > /dev/null &
    ganache_pid=$!
    waitport $ganache_port
}

start_ganache_if_needed() {
    if ganache_running; then
        echo "Using existing ganache instance"
    else
        echo "Starting our own ganache instance"
        start_ganache
    fi
}

start_ganache_if_needed