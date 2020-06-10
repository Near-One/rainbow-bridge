const { exec } = require('child_process');
const path = require('path');

class PrepareCommand {
    static execute (command) {
        var scriptDir = path.resolve(process.cwd(), 'scripts/prepare.sh');

        let shell = [
            'bash',
            scriptDir,
        ].join(' ');

        if (command.bridgeSrc) {
            shell = [shell, '--source', command.bridgeSrc].join(' ');
        }

        if (command.coreSrc) {
            shell = [shell, '--nearcore_source', command.coreSrc].join(' ');
        }

        let env = {};
        for (var e in process.env) {
            env[e] = process.env[e];
        }
        env['LOCAL_BRIDGE_SRC']=command.bridgeSrc;
        env['LOCAL_CORE_SRC']=command.coreSrc;
        env['LOCAL_NEARUP_SRC']=command.nearupSrc;


        var prepareScript = exec(shell, {env: env});
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
