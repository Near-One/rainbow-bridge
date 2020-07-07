const { execSync } = require('child_process');
const ProcessManager = require('pm2');

class CleanCommand {
    static execute () {
        console.log('Stopping all the running processes...');
        ProcessManager.killDaemon((err) => {
            if (err) {
                console.log(`Error stopping pm2 processes. ${err}`);
                process.exit(1);
            }
            try {
                execSync('python3 ~/.rainbow/nearup/main.py stop');
            } catch (err) {
                console.log(`Error stopping nearup ${err}`);
            }
            console.log('Cleaning ~/.rainbow directory');
            execSync('rm -rf ~/.rainbow');
            console.log('Cleaning done...');
            process.exit(0);
        });
    }
}

exports.CleanCommand = CleanCommand;
