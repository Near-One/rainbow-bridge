const { execSync } = require('child_process');

class CleanCommand {
    static execute () {
        console.log('Stopping all the running processess...');
        execSync('pm2 kill && python3 ~/.rainbowup/nearup/nearup stop');
        console.log('Cleaning ~/.rainbowup and ~/.nearup directories...');
        execSync('rm -rf ~/.rainbowup && rm -rf ~/.nearup');
        console.log('Cleaning done...');
    }
}

exports.CleanCommand = CleanCommand;
