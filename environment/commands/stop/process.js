const ProcessManager = require('pm2');

class StopManagedProcessCommand {
    static execute (command) {
        const serviceName = command._name;
        console.log('Stopping process:', serviceName);
        ProcessManager.stop(serviceName, (err) => {
            if (err) {
                console.log('Error stopping the process due to:', err);
                process.exit(1);
            }
            console.log(serviceName, 'successfully stopped...');
            // @ts-ignore
            ProcessManager.disconnect();
        });
    }
}

exports.StopManagedProcessCommand = StopManagedProcessCommand;
