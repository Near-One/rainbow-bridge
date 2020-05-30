const {exec} = require('child_process');

class CleanCommand {
  static execute() { exec('rm -rf ~/.rainbowup && pm2 kill'); }
}

exports.CleanCommand = CleanCommand;
