const { execSync } = require('child_process');
const ProcessManager = require('pm2');

class CleanCommand {
    static execute () {
        console.log('Stopping all the running processess...');
        ProcessManager.killDaemon((err) => {
            if (err) {
                console.log(`Error stopping pm2 processes. ${err}`);
                process.exit(1);
            }
        });
        // @ts-ignore
        ProcessManager.disconnect((err) => {
            if (err) {
                process.exit(1);
            }
        });
        try {
            execSync('python3 ~/.rainbowup/nearup/nearup stop');
        } catch (err) {
            console.log(`Error stopping nerup ${err}`);
        }
        console.log('Cleaning ~/.rainbowup , ~/.nearup , and ~/.near directories...');
        execSync('rm -rf ~/.rainbowup; rm -rf ~/.nearup; rm -rf ~/.near');
        console.log('Cleaning done...');
    }
}

exports.CleanCommand = CleanCommand;
