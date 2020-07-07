const { execSync } = require('child_process');

class StopLocalNearNodeCommand {
    static execute () {
        console.log('Stopping local near node...');
        const command = 'python3 ~/.rainbow/nearup/main.py stop';
        try {
            execSync(command);
        } catch (err) {
            console.log('Error stopping local near node', err);
        }
    }
}

exports.StopLocalNearNodeCommand = StopLocalNearNodeCommand;
