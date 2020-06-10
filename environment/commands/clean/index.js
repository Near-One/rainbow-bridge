const { execSync } = require('child_process');

class CleanCommand {
    static execute () {
        console.log('Stopping all the running processess...');
        try {
            execSync('pm2 kill');
        } catch (err) {
            console.log(`Error stopping pm2 ${err}`);
        }
        try {
            execSync('python3 ~/.rainbowup/nearup/nearup stop');
        } catch (err) {
            console.log(`Error stopping nerup ${err}`);
        }
        console.log('Cleaning ~/.rainbowup and ~/.nearup directories...');
        execSync('rm -rf ~/.rainbowup && rm -rf ~/.nearup');
        console.log('Cleaning done...');
    }
}

exports.CleanCommand = CleanCommand;
