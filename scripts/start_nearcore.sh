#!/usr/bin/env bash

_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

source $_DIR/waitport.sh
source $_DIR/trapadd.sh

# Exit script as soon as a command fails.
set -o errexit

# Executes cleanup function at script exit.
trap_add cleanup_nearcore EXIT

cleanup_nearcore() {
    # Kill the nearcore instance that we started (if we started one and if it's still running).
    if [ -n "$nearcore_started" ]; then
        docker kill nearcore watchtower > /dev/null &
    fi
}

nearcore_port=24567

nearcore_running() {
    nc -z localhost "$nearcore_port"
}

start_nearcore() {
    echo "ethrelay" | "$_DIR/start_localnet.py" --home "$_DIR/.near" --image "nearprotocol/nearcore:ethdenver"
    waitport $nearcore_port
}

start_nearcore_if_needed() {
    if nearcore_running; then
        echo "Using existing nearcore instance"
    else
        echo "Starting our own nearcore instance"
        rm -rf "$_DIR/.near"
        start_nearcore
        nearcore_started=1
    fi
}

start_nearcore_if_needed
