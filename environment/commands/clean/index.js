const { execSync } = require('child_process');
const ProcessManager = require('pm2');

class CleanCommand {
    static execute () {
        console.log('Stopping all the running processess...');
        ProcessManager.connect((err) => {
            if (err) {
                console.log(
                    'Unable to connect to the ProcessManager daemon! Please retry.');
                return;
            }
            ProcessManager.killDaemon((err, processDescription) => {
                if (err) {
                    console.log(`Error killing process ${processDescription} ${err}`);
                }
                ProcessManager.disconnect();
            });
        })

        try {
            execSync('python3 ~/.rainbowup/nearup/nearup stop');
        } catch(err) {
            console.log(`Error stopping nerup ${err}`);
        }
        console.log('Cleaning ~/.rainbowup and ~/.nearup directories...');
        execSync('rm -rf ~/.rainbowup && rm -rf ~/.nearup');
        console.log('Cleaning done...');
    }
}

exports.CleanCommand = CleanCommand;
