const {exec} = require('child_process');
const path = require('path')

class PrepareCommand {
  static execute(command) {
    var scriptDir = path.resolve(process.cwd(), "scripts/prepare.sh");

    let shell = [
      "bash",
      scriptDir,
    ].join(' ');

    if (command.bridgeSrc) {
      shell = [ shell, "--source", command.bridgeSrc ].join(' ');
    }

    if (command.coreSrc) {
      shell = [ shell, "--nearcore_source", command.coreSrc ].join(' ');
    }

    var prepareScript = exec(shell);
    prepareScript.stdout.on(
        'data', function(data) { process.stdout.write(data.toString()); });
    prepareScript.stderr.on(
        'data', function(data) { process.stdout.write(data.toString()); });
  }
}

exports.PrepareCommand = PrepareCommand;
