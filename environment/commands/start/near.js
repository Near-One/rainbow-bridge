const util = require('util');
const { execSync } = require('child_process');
const request = require('request');
const { getLocalNearNodeURL } = require('./helpers');

class StartLocalNearNodeCommand {
    static execute () {
        const command = util.format(
            '~/.rainbowup/nearup/nearup localnet --num-nodes 1 --binary-path %s',
            '~/.rainbowup/core/target/debug',
        );
        request(getLocalNearNodeURL(), { json: true }, (err, res, body) => {
            if (err) {
                console.log(execSync(command).toString());
            } else {
                console.log('Local Node is already running. Skipping...');
            }
        });
    }
}

exports.StartLocalNearNodeCommand = StartLocalNearNodeCommand;
