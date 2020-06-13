const { exec } = require('child_process');
const path = require('path');
const { RainbowConfig } = require('../lib/config');

class PrepareCommand {
    static execute () {
        var scriptDir = path.resolve(process.cwd(), 'scripts/prepare.sh');

        const shell = [
            'bash',
            scriptDir,
        ].join(' ');

        const env = {};
        for (const e in process.env) {
            env[e] = process.env[e];
        }
        env.LOCAL_BRIDGE_SRC = RainbowConfig.getParam('bridge-src');
        env.LOCAL_CORE_SRC = RainbowConfig.getParam('core-src');
        env.LOCAL_NEARUP_SRC = RainbowConfig.getParam('nearup-src');

        var prepareScript = exec(shell, { env: env });
        prepareScript.stdout.on(
            'data', function (data) {
                console.log(data.toString());
            });
        prepareScript.stderr.on(
            'data', function (data) {
                console.log(data.toString());
            });
    }
}

exports.PrepareCommand = PrepareCommand;
