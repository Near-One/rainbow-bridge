const ProcessManager = require('pm2');

class StopManagedProcessCommand {
    static execute (command) {
        const serviceName = command._name;
        console.log('Stopping process:', serviceName);
        ProcessManager.delete(serviceName, (err) => {
            if (err) {
                if (!err.message.includes('process or namespace not found')) {
                    console.log('Error stopping the process due to:', err);
                    process.exit(1);
                } else {
                    console.log(serviceName, 'already stopped');
                }
            } else {
                console.log(serviceName, 'successfully stopped...');
            }
            // @ts-ignore
            ProcessManager.disconnect();
        });
    }
}

exports.StopManagedProcessCommand = StopManagedProcessCommand;
